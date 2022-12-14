use crate::ecs::{ECS, ComponentSystem, EntityId, ComponentTypeId, ComponentType, Component};
use crate::components::{BodyComponent, AnyComponent};

pub struct PhysicsSystem {}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl ComponentSystem for PhysicsSystem {
    fn tick(&self, ecs: &mut ECS, dt: f64) -> Vec<(EntityId, ComponentTypeId, AnyComponent)> {
        let bodies = ecs.get_components_mut::<BodyComponent>();
        let ctid = BodyComponent::member_ctid();
        let mut updates = Vec::new();
        
        for (eid, body) in bodies.into_iter() {
            if body.z > 0.0 {
                let mut updated = body.clone();
                updated.z -= 5.0 * dt;
                updates.push((eid, ctid, updated.into_any()));
            }
        }

        updates
    }
}
