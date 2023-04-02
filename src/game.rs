use super::*;
use collision::*;
use player::*;

// Render constants
const BOUNDARY_COLOR: Rgba<f32> = Rgba::GRAY;
const BOUNDARY_WIDTH: f32 = 5.0;

const PLAYER_LEFT_COLOR: Rgba<f32> = Rgba::GREEN;
const PLAYER_RIGHT_COLOR: Rgba<f32> = Rgba::BLUE;

const BALL_COLOR: Rgba<f32> = Rgba::RED;

// Game constants
const ARENA_SIZE: vec2<f32> = vec2(450.0, 300.0);

const PLAYER_SIZE: vec2<f32> = vec2(10.0, 50.0);
const PLAYER_SPEED: f32 = 100.0;

const BALL_RADIUS: f32 = 5.0;
const BALL_SPEED: f32 = 100.0;
const BALL_START_ANGLE_MIN: f32 = 0.5;
const BALL_START_ANGLE_MAX: f32 = 0.7;

pub struct Game {
    geng: Geng,
    world: World,
    update_schedule: Schedule,
    player_control: PlayerControl,
    render_colliders: bool,
}

impl Game {
    pub fn new(geng: &Geng) -> Self {
        let mut game = Self {
            geng: geng.clone(),
            world: World::new(),
            update_schedule: Schedule::default(),
            player_control: PlayerControl::default(),
            render_colliders: true,
        };
        game.init();
        game
    }

    fn init(&mut self) {
        // Resources
        self.world.insert_resource(Camera(Camera2d {
            center: vec2::ZERO,
            rotation: 0.0,
            fov: 400.0,
        }));
        self.world.insert_resource(TimeRes {
            delta_time: Time::ONE,
            game_time: 0.0,
        });
        self.world.insert_resource(Scores([0; 2]));
        let boundary = Aabb2::ZERO.extend_symmetric(ARENA_SIZE / 2.0);
        self.world.insert_resource(Boundary(boundary));

        // Systems
        self.update_schedule.add_systems((
            control_players,
            movement.after(control_players),
            collisions.after(movement),
            check_finish.after(collisions),
        ));

        // Entities
        let border_translate = boundary.height() / 2.0 + BOUNDARY_WIDTH / 2.0;
        let border = Aabb2::point(boundary.center())
            .extend_symmetric(vec2(boundary.width(), BOUNDARY_WIDTH) / 2.0);
        self.world
            .spawn(Position(vec2(0.0, border_translate)))
            .insert(ColliderType::Block)
            .insert(Collider::Aabb(border))
            .insert(Color(BOUNDARY_COLOR));
        self.world
            .spawn(Position(vec2(0.0, -border_translate)))
            .insert(ColliderType::Block)
            .insert(Collider::Aabb(border))
            .insert(Color(BOUNDARY_COLOR));

        let player_offset = PLAYER_SIZE.x / 2.0 + 5.0;
        self.world
            .spawn(Player::new(0, PLAYER_SPEED))
            .insert(Position(vec2(-ARENA_SIZE.x / 2.0 + player_offset, 0.0)))
            .insert(Velocity(vec2::ZERO))
            .insert(ColliderType::Block)
            .insert(Collider::Aabb(
                Aabb2::ZERO.extend_symmetric(PLAYER_SIZE / 2.0),
            ))
            .insert(Color(PLAYER_LEFT_COLOR));
        self.world
            .spawn(Player::new(1, PLAYER_SPEED))
            .insert(Position(vec2(ARENA_SIZE.x / 2.0 - player_offset, 0.0)))
            .insert(Velocity(vec2::ZERO))
            .insert(ColliderType::Block)
            .insert(Collider::Aabb(
                Aabb2::ZERO.extend_symmetric(PLAYER_SIZE / 2.0),
            ))
            .insert(Color(PLAYER_RIGHT_COLOR));

        self.new_ball();
    }

    /// Creates a new ball at the center of the world and assigns a random velocity to it
    fn new_ball(&mut self) {
        let mut queue = bevy_ecs::system::CommandQueue::from_world(&mut self.world);
        let mut commands = Commands::new(&mut queue, &self.world);
        spawn_ball(&mut commands);
        queue.apply(&mut self.world);
    }

    fn update_control(&mut self) {
        let keys = [
            (geng::Key::W, geng::Key::S),
            (geng::Key::Up, geng::Key::Down),
        ];
        for (control, (up, down)) in std::iter::zip(&mut self.player_control.directions, keys) {
            *control = 0.0;
            if self.geng.window().is_key_pressed(up) {
                *control += 1.0;
            }
            if self.geng.window().is_key_pressed(down) {
                *control -= 1.0;
            }
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        self.update_control();
        let control = std::mem::take(&mut self.player_control);

        self.world.insert_resource(control);
        self.world.resource_mut::<TimeRes>().delta_time = delta_time as f32;

        self.update_schedule.run(&mut self.world);
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);

        let camera = self
            .world
            .get_resource::<Camera>()
            .expect("No camera found")
            .clone();

        let mut query = self.world.query::<&Renderable>();
        for renderable in query.iter(&self.world) {
            self.geng
                .draw_2d(framebuffer, camera.deref(), &renderable.inner)
        }

        if self.render_colliders {
            let mut query = self.world.query::<(&Position, &Collider, Option<&Color>)>();
            for (pos, collider, color) in query.iter(&self.world) {
                let color = color.map(|c| c.0).unwrap_or(Rgba::new(1.0, 0.0, 0.0, 0.5));
                let collider = collider.at(pos.0);
                match collider {
                    Collider::Circle(circle) => self.geng.draw_2d(
                        framebuffer,
                        camera.deref(),
                        &draw_2d::Ellipse::circle(circle.center, circle.radius, color),
                    ),
                    Collider::Aabb(aabb) => self.geng.draw_2d(
                        framebuffer,
                        camera.deref(),
                        &draw_2d::Quad::new(aabb, color),
                    ),
                }
            }
        }

        // Display scores in format: "00 - 00"
        let scores = self
            .world
            .get_resource::<Scores>()
            .expect("No scores found");
        let boundary = self
            .world
            .get_resource::<Boundary>()
            .expect("No boundary found");
        let scores = format!("{:02} - {:02}", scores[0], scores[1]);
        self.geng.default_font().draw(
            framebuffer,
            camera.deref(),
            &scores,
            vec2(0.0, boundary.max.y + 10.0), // Just above the top boundary
            geng::TextAlign::CENTER,
            32.0,
            Rgba::WHITE,
        );
    }
}

fn control_players(control: Res<PlayerControl>, mut query: Query<(&Player, &mut Velocity)>) {
    for (player, mut velocity) in &mut query {
        let control = control.directions[player.id];
        velocity.0 = vec2::UNIT_Y * control * player.speed;
    }
}

fn movement(time: Res<TimeRes>, mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in &mut query {
        pos.0 += vel.0 * time.delta_time;
    }
}

fn collisions(
    mut colliders: Query<(
        &Collider,
        &ColliderType,
        &mut Position,
        Option<&mut Velocity>,
    )>,
) {
    let mut combinations = colliders.iter_combinations_mut();
    while let Some([(a_col, a_type, mut a_pos, a_vel), (b_col, b_type, mut b_pos, b_vel)]) =
        combinations.fetch_next()
    {
        let a_col = a_col.at(a_pos.0);
        let b_col = b_col.at(b_pos.0);
        if let Some(collision) = a_col.collide(b_col) {
            let (a_coef, b_coef) = match (a_type, b_type) {
                (ColliderType::Block, ColliderType::Block) => {
                    match (a_vel.is_some(), b_vel.is_some()) {
                        (false, false) => continue,
                        (false, true) => (0.0, 1.0),
                        (true, false) => (1.0, 0.0),
                        (true, true) => (0.5, 0.5),
                    }
                }
                (ColliderType::Block, ColliderType::Actor) => (0.0, 1.0),
                (ColliderType::Actor, ColliderType::Block) => (1.0, 0.0),
                (ColliderType::Actor, ColliderType::Actor) => (0.5, 0.5),
            };
            let offset = collision.normal * collision.penetration;
            a_pos.0 -= offset * a_coef;
            b_pos.0 += offset * b_coef;

            let bounce = |ty: &ColliderType, mut vel: Mut<Velocity>| {
                let bounciness = match ty {
                    ColliderType::Block => 0.0,
                    ColliderType::Actor => 1.0,
                };
                let proj = collision.normal * vec2::dot(vel.0, collision.normal);
                vel.0 -= proj * (bounciness + 1.0);
            };
            if let Some(vel) = a_vel {
                bounce(a_type, vel);
            }
            if let Some(vel) = b_vel {
                bounce(b_type, vel);
            }
        }
    }
}

fn check_finish(
    mut commands: Commands,
    mut scores: ResMut<Scores>,
    boundary: Res<Boundary>,
    balls: Query<(Entity, &Position), With<Ball>>,
) {
    let mut scored = Vec::new();
    for (e, pos) in &balls {
        if pos.x > boundary.max.x {
            // Left player scored
            scores[0] += 1;
            scored.push(e);
        } else if pos.x < boundary.min.x {
            // Right player scored
            scores[1] += 1;
            scored.push(e);
        }
    }

    for e in scored {
        commands.entity(e).despawn();
        spawn_ball(&mut commands);
    }
}

/// Creates a new ball at the center of the world and assigns a random velocity to it.
fn spawn_ball(commands: &mut Commands) {
    // Generate a random velocity
    let angle_range = BALL_START_ANGLE_MAX - BALL_START_ANGLE_MIN;
    let random_angle = rand::thread_rng().gen_range(0.0..=angle_range * 4.0);

    // Pick side to shoot: 1.0 - right, -1.0 - left
    let horizontal_mult = (random_angle / angle_range / 2.0).floor() * 2.0 - 1.0;
    // Pick vertical direction to shoot: 1.0 - up, -1.0 - down
    let quarter = (random_angle / angle_range).floor() as i32;
    let vertical_mult = (quarter % 2 * 2 - 1) as f32;

    // Generate random direction
    let angle = BALL_START_ANGLE_MIN + random_angle - (quarter as f32) * angle_range;
    let (sin, cos) = angle.sin_cos();
    let direction = vec2(cos * horizontal_mult, sin * vertical_mult);

    let velocity = direction * BALL_SPEED;

    commands
        .spawn(Ball)
        .insert(Position(vec2::ZERO))
        .insert(Velocity(velocity))
        .insert(ColliderType::Actor)
        .insert(Collider::Circle(CircleCollider {
            center: vec2::ZERO,
            radius: BALL_RADIUS,
        }))
        .insert(Color(BALL_COLOR));
}
