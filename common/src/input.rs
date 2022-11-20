use serde::{Serialize, Deserialize};

use crate::components::BodyComponent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    CreateEntity(BodyComponent)
}
 