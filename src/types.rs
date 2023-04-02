use super::*;

pub type Time = f32;
pub type Coord = f32;

#[derive(Debug, Clone, Resource)]
pub struct PlayerControl {
    pub directions: [Coord; 2],
}

#[derive(Debug, Clone, Resource)]
pub struct TimeRes {
    pub delta_time: Time,
    pub game_time: Time,
}

#[derive(Debug, Clone, Resource, Deref, DerefMut)]
pub struct Scores(
    #[deref]
    #[deref_mut]
    pub [u32; 2],
);

#[derive(Debug, Clone, Resource, Deref, DerefMut)]
pub struct Camera(
    #[deref]
    #[deref_mut]
    pub Camera2d,
);

#[derive(Debug, Clone, Copy, Resource, Deref, DerefMut)]
pub struct Boundary(
    #[deref]
    #[deref_mut]
    pub Aabb2<Coord>,
);

#[derive(Debug, Clone, Component)]
pub struct Ball;

#[derive(Component)]
pub struct Renderable {
    pub inner: Box<dyn geng::draw_2d::Draw2d + Send + Sync>,
}

#[derive(Debug, Clone, Copy, Component, Deref, DerefMut)]
pub struct Position(
    #[deref]
    #[deref_mut]
    pub vec2<Coord>,
);

#[derive(Debug, Clone, Copy, Component, Deref, DerefMut)]
pub struct Velocity(
    #[deref]
    #[deref_mut]
    pub vec2<Coord>,
);

#[derive(Debug, Clone, Copy, Component, Deref, DerefMut)]
pub struct Color(
    #[deref]
    #[deref_mut]
    pub Rgba<f32>,
);

impl Default for PlayerControl {
    fn default() -> Self {
        Self {
            directions: [0.0; 2],
        }
    }
}
