use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
pub struct Object {
    id: i64,
}

impl Object {
    pub fn new(id: i64) -> Self {
        Object { id }
    }

    pub fn new8000() -> Self {
        Self::new(8000)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Combo {
    A,
    B,
    C,
}