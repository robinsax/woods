use serde::{Serialize, Deserialize};

use crate::ecs::{Component, ComponentTypeId, ComponentType};

pub const COMPONENT_COUNT: usize = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnyComponent {
    Body(BodyComponent)
}

impl AnyComponent {
    pub fn into_dyn(self) -> Box<dyn Component> {
        match self {
            AnyComponent::Body(inner) => Box::new(inner)
        }
    }
}

macro_rules! assign_component {
    ($t: ty, $i: literal, $($v: tt)*) => {
        impl Component for $t {
            fn ctid(&self) -> ComponentTypeId {
                $i
            }

            fn into_any(self) -> AnyComponent {
                $($v)*(self)
            }

            fn as_any(&self) -> AnyComponent {
                self.clone().into_any()
            }
        }

        impl ComponentType for $t {
            fn member_ctid() -> ComponentTypeId {
                $i
            }
        }
    };
}

// TODO: Where does this live?
pub fn any_components_to_dyn(anys: Vec<AnyComponent>) -> Vec<Box<dyn Component>> {
    let mut dyns = Vec::new();
    for any in anys.into_iter() {
        dyns.push(any.into_dyn());
    }

    dyns
}

pub fn any_components_to_dyn_layout(anys: Vec<AnyComponent>) -> Vec<Option<Box<dyn Component>>> {
    let mut dyns = [None; COMPONENT_COUNT];
    for any in anys.into_iter() {
        let dyn_component = any.into_dyn();
        let ctid = dyn_component.ctid();
        dyns[ctid] = Some(dyn_component);
    }

    Vec::from(dyns)
}

pub fn dyn_components_to_any(dyns: Vec<Option<&dyn Component>>) -> Vec<AnyComponent> {
    let mut anys = Vec::new();
    for dyn_opt in dyns.into_iter() {
        if let Some(component) = dyn_opt {
            // NOTE: Learn why move is illegal here.
            anys.push(component.as_any());
        }
    }
    
    anys
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BodyComponent {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub sx: f64,
    pub sy: f64,
    pub sz: f64
}

assign_component!(BodyComponent, 1, AnyComponent::Body);
