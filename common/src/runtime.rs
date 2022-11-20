use serde::{Serialize, Deserialize};
use log::info;

use crate::components::{AnyComponent, any_components_to_dyn};
use crate::input::Input;
use crate::ecs::{ECS, EntityId, Component, ComponentTypeId, ComponentSystem};
use crate::systems::PhysicsSystem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeMessage {
    NeedLoad,
    Load(Vec<(EntityId, Vec<AnyComponent>)>),
    Input(Input),
    EntityCreate(EntityId, Vec<AnyComponent>),
    ComponentUpdate(EntityId, ComponentTypeId, AnyComponent)
}

#[derive(PartialEq, Debug)]
pub enum RuntimeRole {
    Master,
    Client
}

pub trait RuntimeIo
where
    Self: Send + Sync
{
    fn rx(&self) -> (Vec<Input>, Vec<RuntimeMessage>);
    fn tx(&self, message: RuntimeMessage);
}

pub struct Runtime {
    io: &'static dyn RuntimeIo,
    ecs: ECS,
    role: RuntimeRole,
    systems: Vec<Box<dyn ComponentSystem>>
}

impl Runtime {
    pub fn new(io: &'static dyn RuntimeIo, role: RuntimeRole) -> Self {
        let mut systems: Vec<Box<dyn ComponentSystem>> = Vec::new();
        systems.push(Box::new(PhysicsSystem::new()));

        if role == RuntimeRole::Client {
            info!("request load");
            io.tx(RuntimeMessage::NeedLoad);
        }

        Self {
            io,
            role,
            systems,
            ecs: ECS::new()
        }
    }

    pub fn systems_tick(&mut self, dt: f64) {
        let mut updates = Vec::new();
        for system in self.systems.iter() {
            let system_updates = system.tick(&mut self.ecs, dt);

            updates.extend(system_updates);
        }

        for (eid, ctid, component) in updates {
            let message = RuntimeMessage::ComponentUpdate(eid, ctid, component);

            self.apply_message(message.clone());

            if self.role == RuntimeRole::Master {
                self.io.tx(message);
            }
        }
    }

    pub fn io_tick(&mut self) {
        let (inputs, messages) = self.io.rx();

        for message in messages.into_iter() {
            self.apply_message(message);
        }

        for input in inputs.into_iter() {
            // TODO: Messy clones.
            let message = self.process_input(input.clone());

            self.apply_message(message.clone());

            if self.role == RuntimeRole::Client {
                self.io.tx(RuntimeMessage::Input(input));
            }
        }
    }

    fn process_input(&mut self, input: Input) -> RuntimeMessage {
        match input {
            Input::CreateEntity(position) => {
                let eid = self.ecs.reserve_id();

                RuntimeMessage::EntityCreate(eid, Vec::from([position.into_any()]))
            }
        }
    }

    // TODO: Roll based control (and other validation obviously).
    fn apply_message(&mut self, message: RuntimeMessage) {
        match message {
            RuntimeMessage::Input(input) => {
                // TODO: Flow is super messed up.
                let resultant = self.process_input(input);

                self.apply_message(resultant);
            },
            RuntimeMessage::NeedLoad => {
                let eids: Vec<&usize> = self.ecs.live_eids().collect();
                let mut provision = Vec::with_capacity(eids.len());
                
                for eid in eids {
                    provision.push((eid.to_owned(), self.ecs.get_entity_anys(eid.to_owned())));
                }

                self.io.tx(RuntimeMessage::Load(provision));
            },
            RuntimeMessage::Load(entities) => {
                for (eid, any_components) in entities.into_iter() {
                    self.ecs.create_entity(eid, any_components_to_dyn(any_components))
                }
            },
            RuntimeMessage::EntityCreate(eid, components) => {
                self.ecs.create_entity(eid, any_components_to_dyn(components));
            },
            RuntimeMessage::ComponentUpdate(eid, ctid, component) => {
                self.ecs.update_component(eid, ctid, component.into_dyn());
            }
        }
    }

    pub fn ecs(&self) -> &ECS {
        &self.ecs
    }
}
