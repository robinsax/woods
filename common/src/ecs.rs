use std::{fmt::Debug, collections::LinkedList};
use std::collections::HashMap;

use crate::components::{COMPONENT_COUNT, AnyComponent, dyn_components_to_any};

pub type EntityId = usize;
pub type ComponentTypeId = usize;

pub trait ComponentType {
    fn member_ctid() -> ComponentTypeId;
}

pub trait Component
where
    Self: Debug + Sync + Send
{
    fn ctid(&self) -> ComponentTypeId;
    fn into_any(self) -> AnyComponent;
    fn as_any(&self) -> AnyComponent;
}

pub trait ComponentSystem
where
    Self: Sync + Send
{
    fn tick(&self, ecs: &mut ECS, dt: f64) -> Vec<(EntityId, ComponentTypeId, AnyComponent)>;
}

#[derive(Debug)]
pub struct ECS {
    // TODO: Better arena without a hash table.
    components: HashMap<ComponentTypeId, HashMap<EntityId, Option<Box<dyn Component>>>>,
    next_id: EntityId,
    live_entities: LinkedList<EntityId>
}

impl ECS {
    pub fn new() -> Self {
        let mut components = HashMap::new();
        for k in 0..COMPONENT_COUNT + 1 {
            components.insert(k, HashMap::new());
        }

        Self {
            components,
            live_entities: LinkedList::new(),
            next_id: 1
        }
    }

    pub fn reserve_id(&mut self) -> EntityId {
        // TODO: Many related races and O(n) lookup.
        loop {
            self.next_id += 1;

            if !self.live_entities.contains(&self.next_id) {
                break;
            }
        }
        
        self.next_id
    }

    pub fn live_eids(&self) -> impl Iterator<Item = &EntityId> {
        self.live_entities.iter()
    }

    pub fn create_entity(&mut self, eid: EntityId, components: Vec<Box<dyn Component>>) {        
        for component in components.into_iter() {
            let ctid = component.ctid();

            self.live_entities.push_back(eid);

            let component_set = self.components.get_mut(&ctid).unwrap();
            component_set.insert(eid, Some(component));
        }
    }

    pub fn get_entity_anys(&self, eid: EntityId) -> Vec<AnyComponent> {
        dyn_components_to_any(self.get_entity(eid))
    }

    pub fn get_entity(&self, eid: EntityId) -> Vec<Option<&dyn Component>> {
        let mut entity = [None; COMPONENT_COUNT];

        for (ctid, component_list) in self.components.iter() {
            if let Some(Some(component)) = component_list.get(&eid) {
                entity[ctid.to_owned() - 1] = Some(component.as_ref());
            }
        }

        Vec::from(entity)
    }

    pub fn get_components_mut<T>(&mut self) -> Vec<(EntityId, &mut T)>
    where
        T: ComponentType
    {
        let ctid = T::member_ctid();

        let mut refs = Vec::new();
        for (eid, component) in self.components.get_mut(&ctid).unwrap() {
            if let Some(cell) = component {
                // SAFETY: Correct entity type invariant per write.
                let typed_cell = unsafe {
                    &mut *(cell as *mut Box<dyn Component> as *mut Box<T>)
                };

                refs.push((eid.to_owned(), typed_cell.as_mut()));
            }
        }

        refs
    }

    pub fn get_components<T>(&self) -> Vec<(EntityId, &T)>
    where
        T: ComponentType
    {
        let ctid = T::member_ctid();

        let mut refs = Vec::new();
        for (eid, component) in self.components.get(&ctid).unwrap() {
            if let Some(cell) = component {
                // SAFETY: Correct entity type invariant per write.
                let typed_cell = unsafe {
                    &*(cell as *const Box<dyn Component> as *const Box<T>)
                };

                refs.push((eid.to_owned(), typed_cell.as_ref()));
            }
        }

        refs
    }

    pub fn get_component<T>(&self, eid: EntityId) -> Option<&T>
    where
        T: ComponentType
    {
        let ctid = T::member_ctid();

        match self.components.get(&ctid).unwrap().get(&eid) {
            Some(Some(cell)) => {
                // SAFETY: Correct entity type invariant per write.
                let typed_cell = unsafe {
                    &*(cell as *const Box<dyn Component> as *const Box<T>)
                };

                Some(typed_cell.as_ref())
            },
            _ => None
        }
    }

    pub fn update_component(&mut self, eid: EntityId, ctid: ComponentTypeId, component: Box<dyn Component>) {
        self.components.get_mut(&ctid).unwrap().insert(eid, Some(component));
    }
}
