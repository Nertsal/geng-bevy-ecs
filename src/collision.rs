use super::*;

#[derive(Debug, Clone, Copy, Component)]
pub enum ColliderType {
    Block,
    Actor,
}

#[derive(Debug, Clone, Copy, Component)]
pub enum Collider {
    Circle(CircleCollider),
    Aabb(Aabb2<Coord>),
}

#[derive(Debug, Clone, Copy)]
pub struct CircleCollider {
    pub center: vec2<Coord>,
    pub radius: Coord,
}

#[derive(Debug, Clone, Copy)]
pub struct Collision {
    pub normal: vec2<f32>,
    pub penetration: f32,
}

impl Collider {
    pub fn at(self, pos: vec2<Coord>) -> Self {
        match self {
            Self::Circle(circle) => Self::Circle(CircleCollider {
                center: circle.center + pos,
                ..circle
            }),
            Self::Aabb(aabb) => Self::Aabb(aabb.translate(pos)),
        }
    }

    pub fn collide(self, other: Self) -> Option<Collision> {
        match (self, other) {
            (Self::Circle(a), Self::Circle(b)) => collide_circle(a, b),
            (Self::Circle(circle), Self::Aabb(aabb)) => {
                collide_aabb_circle(aabb, circle).map(|collision| Collision {
                    normal: -collision.normal,
                    ..collision
                })
            }
            (Self::Aabb(aabb), Self::Circle(circle)) => collide_aabb_circle(aabb, circle),
            (Self::Aabb(a), Self::Aabb(b)) => collide_aabb(a, b),
        }
    }
}

fn collide_circle(a: CircleCollider, b: CircleCollider) -> Option<Collision> {
    let delta = b.center - a.center;
    let dist = delta.len();
    let normal = if dist.approx_eq(&0.0) {
        vec2::ZERO
    } else {
        delta / dist
    };
    let penetration = a.radius + b.radius - dist;
    (penetration > 0.0).then_some(Collision {
        normal,
        penetration,
    })
}

fn collide_aabb(a: Aabb2<Coord>, b: Aabb2<Coord>) -> Option<Collision> {
    if !a.intersects(&b) {
        return None;
    }

    let dx_right = a.max.x - b.min.x;
    let dx_left = b.max.x - a.min.x;
    let dy_up = a.max.y - b.min.y;
    let dy_down = b.max.y - a.min.y;

    let (nx, px) = if dx_right < dx_left {
        (Coord::ONE, dx_right)
    } else {
        (-Coord::ONE, dx_left)
    };
    let (ny, py) = if dy_up < dy_down {
        (Coord::ONE, dy_up)
    } else {
        (-Coord::ONE, dy_down)
    };

    if px <= 0.0 || py <= 0.0 {
        None
    } else if px < py {
        Some(Collision {
            normal: vec2::UNIT_X * nx,
            penetration: px,
            // offset_dir: vec2::UNIT_Y * ny,
            // offset: py,
        })
    } else {
        Some(Collision {
            normal: vec2::UNIT_Y * ny,
            penetration: py,
            // offset_dir: vec2::UNIT_X * nx,
            // offset: px,
        })
    }
}

fn collide_aabb_circle(aabb: Aabb2<Coord>, circle: CircleCollider) -> Option<Collision> {
    let dx = circle.center.x - aabb.min.x;
    let dy = circle.center.y - aabb.min.y;
    let aabb_size = aabb.size();

    if dx >= 0.0 && dx <= aabb_size.x {
        if dy <= 0.0 && dy >= -circle.radius {
            // Bottom
            Some(Collision {
                normal: vec2(0.0, -1.0),
                penetration: dy + circle.radius,
            })
        } else if dy >= aabb_size.y && dy <= aabb_size.y + circle.radius {
            // Top
            Some(Collision {
                normal: vec2(0.0, 1.0),
                penetration: aabb_size.y + circle.radius - dy,
            })
        } else {
            None
        }
    } else if dy >= 0.0 && dy <= aabb_size.y {
        if dx <= 0.0 && dx >= -circle.radius {
            // Left
            Some(Collision {
                normal: vec2(-1.0, 0.0),
                penetration: dx + circle.radius,
            })
        } else if dx >= aabb_size.x && dx <= aabb_size.x + circle.radius {
            // Right
            Some(Collision {
                normal: vec2(1.0, 0.0),
                penetration: aabb_size.x + circle.radius - dx,
            })
        } else {
            None
        }
    } else {
        let corner = if dx <= 0.0 {
            if dy <= 0.0 {
                aabb.bottom_left()
            } else {
                aabb.top_left()
            }
        } else if dy <= 0.0 {
            aabb.bottom_right()
        } else {
            aabb.top_right()
        };
        let normal = circle.center - corner;
        let penetration = circle.radius - normal.len();
        let normal = normal.normalize();
        if penetration >= 0.0 {
            Some(Collision {
                normal,
                penetration,
            })
        } else {
            None
        }
    }
}
