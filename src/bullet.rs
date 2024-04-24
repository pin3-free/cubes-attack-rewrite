use std::fmt::Debug;
use std::marker::PhantomData;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_xpbd_2d::{math::AdjustPrecision, prelude::*};
use leafwing_input_manager::prelude::*;

use rand::Rng;

use crate::prelude::*;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileHitEvent<Enemy>>()
            .add_event::<ProjectileHitEvent<Player>>()
            .insert_resource(CursorPosition::default())
            .add_event::<ShootEvent>()
            .add_systems(
                Update,
                (
                    cursor_position_system,
                    shoot_input,
                    bullet_spawner,
                    move_bullets,
                    emit_projectile_hits::<Enemy>,
                    emit_projectile_hits::<Player>,
                    tick_shot,
                    expire_bullets,
                )
                    .chain(),
            );
    }
}

#[derive(Component, Debug)]
pub struct Projectile;

#[derive(Component, Debug, Clone, Copy)]
pub struct ProjectileDamage(pub f32);

#[derive(Bundle, Debug)]
pub struct ProjectileBundle {
    collider: Collider,
    collision_layers: CollisionLayers,
    lifetime: BulletLifetimeTimer,
    movement: MovementBundle,
    projectile: Projectile,
    rigid_body: RigidBody,
    sensor: Sensor,
    shot_direction: ShotDirection,
    damage: ProjectileDamage,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ShotDirection(Vec2);

#[derive(Component, Clone, Copy)]
pub struct ShotLocation(Vec2);

#[derive(Component, Debug)]
pub struct BulletLifetimeTimer(Timer);

#[derive(PhysicsLayer, Debug, Clone, Copy)]
pub enum GameLayer {
    Player,
    Enemy,
    Bullet,
    XpCrumb,
    HealingCrumb,
}

impl ProjectileBundle {
    pub fn new(
        acceleration: MovementAcceleration,
        collider: Collider,
        shot_direction: ShotDirection,
        collision_layers: CollisionLayers,
        damage: ProjectileDamage,
    ) -> Self {
        Self {
            projectile: Projectile,
            collider,
            shot_direction,
            movement: MovementBundle::new(acceleration.0, 1.),
            lifetime: BulletLifetimeTimer(Timer::from_seconds(4., TimerMode::Once)),
            rigid_body: RigidBody::Dynamic,
            sensor: Sensor,
            collision_layers,
            damage,
        }
    }
}

#[derive(Component)]
pub struct ShootCooldown(Timer);

impl Default for ShootCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(0.25, TimerMode::Once))
    }
}

#[derive(Event)]
pub struct ShootEvent {
    shot_location: ShotLocation,
    shot_direction: ShotDirection,
    acceleration: MovementAcceleration,
    collision_layers: CollisionLayers,
    damage: ProjectileDamage,
}

#[derive(Resource, Default)]
struct CursorPosition(Vec2);

fn cursor_position_system(
    mut cursor_position: ResMut<CursorPosition>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if let (Ok((camera, camera_transform)), Ok(window)) =
        (q_camera.get_single(), q_window.get_single())
    {
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            cursor_position.0 = world_position;
        }
    }
}

fn shoot_input(
    query: Query<
        (
            &ActionState<Action>,
            Option<&ShootCooldown>,
            &Transform,
            Entity,
        ),
        With<Player>,
    >,
    cursor_position: Res<CursorPosition>,
    mut commands: Commands,
    mut ev_writer: EventWriter<ShootEvent>,
) {
    if let Ok((action_state, shoot_cooldown, transform, entity)) = query.get_single() {
        let shot_direction = cursor_position.0 - transform.translation.truncate();
        let accuracy_angle = std::f32::consts::PI / 36.;
        let shot_displace = rand::thread_rng().gen_range(-accuracy_angle..accuracy_angle);
        let displaced_direction = Quat::from_rotation_z(shot_displace)
            .mul_vec3(shot_direction.extend(0.))
            .truncate();
        if action_state.pressed(&Action::Shoot) && shoot_cooldown.is_none() {
            ev_writer.send(ShootEvent {
                shot_location: ShotLocation(transform.translation.xy()),
                shot_direction: ShotDirection(displaced_direction.normalize()),
                collision_layers: CollisionLayers::new(GameLayer::Bullet, [GameLayer::Enemy]),
                acceleration: MovementAcceleration(30000.),
                damage: ProjectileDamage(5.),
            });

            commands.entity(entity).insert(ShootCooldown::default());
        }
    }
}

fn tick_shot(
    time: Res<Time>,
    mut query: Query<(&mut ShootCooldown, Entity)>,
    mut commands: Commands,
) {
    query.iter_mut().for_each(|(mut cooldown, entity)| {
        if cooldown.0.tick(time.delta()).finished() {
            commands.entity(entity).remove::<ShootCooldown>();
        }
    })
}

fn expire_bullets(
    time: Res<Time>,
    mut query: Query<(&mut BulletLifetimeTimer, Entity)>,
    mut commands: Commands,
) {
    query.iter_mut().for_each(|(mut lifetime_timer, entity)| {
        if lifetime_timer.0.tick(time.delta()).finished() {
            commands.entity(entity).despawn_recursive();
        }
    })
}

fn move_bullets(
    time: Res<Time>,
    mut controllers: Query<
        (&MovementAcceleration, &mut LinearVelocity, &ShotDirection),
        With<Projectile>,
    >,
) {
    let delta_time = time.delta_seconds_f64().adjust_precision();
    controllers.iter_mut().for_each(
        |(acceleration, mut velocity, ShotDirection(Vec2 { x, y }))| {
            velocity.x = x * acceleration.0 * delta_time;
            velocity.y = y * acceleration.0 * delta_time;
        },
    )
}

fn bullet_spawner(mut ev_reader: EventReader<ShootEvent>, mut commands: Commands) {
    ev_reader.read().for_each(
        |ShootEvent {
             shot_location,
             shot_direction,
             collision_layers,
             acceleration,
             damage,
         }| {
            let bullet = ProjectileBundle::new(
                *acceleration,
                Collider::rectangle(5., 5.),
                *shot_direction,
                *collision_layers,
                *damage,
            );
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::YELLOW,
                        custom_size: Some(Vec2::splat(4.)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(shot_location.0.x, shot_location.0.y, 0.)
                        .with_rotation(Quat::from_rotation_arc(
                            Vec3::Y,
                            shot_direction.0.extend(0.),
                        )),
                    ..Default::default()
                },
                bullet,
            ));
        },
    )
}

#[derive(Event, Debug)]
pub struct ProjectileHitEvent<T: Component + Debug> {
    pub projectile: Entity,
    pub target: Entity,
    marker: PhantomData<T>,
}

impl<T: Component + Debug> ProjectileHitEvent<T> {
    fn new(projectile: Entity, target: Entity) -> Self {
        Self {
            projectile,
            target,
            marker: Default::default(),
        }
    }
}

fn emit_projectile_hits<T: Component + Debug>(
    hit_targets: Query<(Entity, &CollidingEntities), With<Projectile>>,
    mut ev_writer: EventWriter<ProjectileHitEvent<T>>,
) {
    hit_targets.iter().for_each(|(entity, colliding_entities)| {
        if !colliding_entities.0.is_empty() {
            colliding_entities.0.iter().for_each(|target| {
                ev_writer.send(ProjectileHitEvent::new(entity, *target));
            });
        }
    });
}
