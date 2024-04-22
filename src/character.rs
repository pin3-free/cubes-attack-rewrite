use bevy::prelude::*;
use bevy_xpbd_2d::{
    math::{AdjustPrecision, Scalar},
    prelude::*,
};
use leafwing_input_manager::prelude::*;

pub struct CharacterControllerPlugin;

use crate::prelude::Player;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default())
            .insert_resource(PlayerPosition(Vec2::ZERO))
            .add_event::<PlayerMoveEvent>()
            .add_systems(
                Update,
                (
                    movement_input,
                    process_pushed,
                    movement,
                    apply_movement_damping,
                    update_player_position,
                    tick_dash_cooldown,
                )
                    .chain(),
            );
    }
}

#[derive(Resource)]
pub struct PlayerPosition(pub Vec2);

fn update_player_position(
    q_player: Query<&Transform, (With<Player>, Changed<Transform>)>,
    mut player_pos: ResMut<PlayerPosition>,
) {
    if let Ok(player_tr) = q_player.get_single() {
        player_pos.0 = player_tr.translation.truncate();
    }
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    Move,
    Dash,
    Shoot,
}

impl Action {
    pub fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        input_map.insert(Action::Move, VirtualDPad::wasd());
        input_map.insert(Action::Dash, KeyCode::ShiftLeft);
        input_map.insert(Action::Shoot, MouseButton::Left);

        input_map
    }
}

#[derive(Event)]
enum PlayerMoveEvent {
    Move(Vec2),
    Dash(Vec2),
}

#[derive(Component)]
pub struct CharacterController;

#[derive(Debug, Component, Clone, Copy)]
pub struct MovementAcceleration(pub Scalar);

#[derive(Debug, Component)]
pub struct MovementDampingFactor(pub Scalar);

#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    movement: MovementBundle,
    input_management: InputManagerBundle<Action>,
}

#[derive(Bundle, Debug)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
}

impl MovementBundle {
    pub const fn new(acceleration: Scalar, damping: Scalar) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            damping: MovementDampingFactor(damping),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(1250.0, 0.9)
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider, input_map: InputMap<Action>) -> Self {
        Self {
            character_controller: CharacterController,
            rigid_body: RigidBody::Dynamic,
            collider,
            movement: MovementBundle::default(),
            input_management: InputManagerBundle::with_map(input_map),
        }
    }

    #[allow(dead_code)]
    pub fn with_movement(mut self, acceleration: Scalar, damping: Scalar) -> Self {
        {
            self.movement = MovementBundle::new(acceleration, damping);
            self
        }
    }
}

fn movement_input(
    query: Query<(&ActionState<Action>, Option<&DashCooldown>, Entity), With<Player>>,
    mut commands: Commands,
    mut ev_writer: EventWriter<PlayerMoveEvent>,
) {
    if let Ok((action_state, dash_cooldown, entity)) = query.get_single() {
        if action_state.pressed(&Action::Move) {
            let dir = action_state.clamped_axis_pair(&Action::Move).unwrap().xy();
            ev_writer.send(PlayerMoveEvent::Move(dir));

            if action_state.just_pressed(&Action::Dash) && dash_cooldown.is_none() {
                ev_writer.send(PlayerMoveEvent::Dash(dir));
                commands.entity(entity).insert(DashCooldown::default());
            }
        }
    }
}

fn tick_dash_cooldown(
    time: Res<Time>,
    mut query: Query<(&mut DashCooldown, Entity)>,
    mut commands: Commands,
) {
    query.iter_mut().for_each(|(mut cooldown, entity)| {
        if cooldown.0.tick(time.delta()).finished() {
            commands.entity(entity).remove::<DashCooldown>();
        }
    })
}

#[derive(Component)]
struct DashCooldown(Timer);

impl Default for DashCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Once))
    }
}

#[derive(Component)]
pub struct PushDirection(Vec2);

#[derive(Component)]
pub struct Pushed {
    direction: PushDirection,
    acceleration: MovementAcceleration,
}

impl Pushed {
    fn get_push_vector(&self) -> Vec2 {
        self.direction.0 * self.acceleration.0
    }
}

impl Pushed {
    pub fn new(direction: Vec2, acceleration: f32) -> Self {
        Self {
            direction: PushDirection(direction),
            acceleration: MovementAcceleration(acceleration),
        }
    }
}

fn process_pushed(
    mut pushed_q: Query<(&mut LinearVelocity, &Pushed, Entity)>,
    mut commands: Commands,
) {
    pushed_q
        .iter_mut()
        .for_each(|(mut velocity, pushed, entity)| {
            let push_vector = pushed.get_push_vector();
            velocity.x += push_vector.x;
            velocity.y += push_vector.y;
            commands.entity(entity).remove::<Pushed>();
        })
}

fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<PlayerMoveEvent>,
    mut controllers: Query<(&MovementAcceleration, &mut LinearVelocity), With<Player>>,
) {
    let delta_time = time.delta_seconds_f64().adjust_precision();
    movement_event_reader.read().for_each(|event| {
        controllers
            .iter_mut()
            .for_each(|(acceleration, mut velocity)| match event {
                PlayerMoveEvent::Move(Vec2 { x, y }) => {
                    velocity.x += x * acceleration.0 * delta_time;
                    velocity.y += y * acceleration.0 * delta_time;
                }
                PlayerMoveEvent::Dash(Vec2 { x, y }) => {
                    velocity.x += x * acceleration.0 * delta_time * 25.;
                    velocity.y += y * acceleration.0 * delta_time * 25.;
                }
            })
    })
}

fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    query.iter_mut().for_each(|(damping, mut velocity)| {
        velocity.x *= damping.0;
        velocity.y *= damping.0;
    })
}
