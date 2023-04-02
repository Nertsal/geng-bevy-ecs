use super::*;

#[derive(Debug, Clone, Component)]
pub struct Player {
    pub id: usize,
    pub speed: f32,
}

impl Player {
    pub fn new(id: usize, speed: f32) -> Self {
        Self { id, speed }
    }
}
