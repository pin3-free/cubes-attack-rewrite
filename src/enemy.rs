use std::{ops::Div, time::Duration};

use bevy::prelude::*;
use bevy_xpbd_2d::{math::AdjustPrecision, prelude::*};
use rand::Rng;

use crate::{
    bullet::GameLayer,
    character::{MovementAcceleration, MovementBundle, PlayerPosition, Pushed},
    hurtbox::{DamageTaken, Dead, Hurt, HurtboxBundle},
    xp_crumbs::XpCrumbBundle,
    Enemy, Player,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyHealthScaling(1.))
            .insert_resource(EnemySpawner::default())
            .add_systems(
                Update,
                (
                    enemy_on_hurt_system,
                    enemy_on_dead_system,
                    spawn_enemies,
                    update_enemy_health_scaling,
                    update_spawner_timer,
                    push_player_on_contact,
                    damage_player_on_contact,
                    tick_invulnerable,
                    move_enemies_system,
                )
                    .chain(),
            );
    }
}

#[derive(Resource)]
struct EnemySpawner {
    min_delay: Duration,
    max_delay: Duration,
    timer: Timer,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self::new(1., 2.)
    }
}

#[derive(Resource)]
struct EnemyHealthScaling(f32);

fn update_enemy_health_scaling(time: Res<Time>, mut scaling: ResMut<EnemyHealthScaling>) {
    scaling.0 = (1. as f32).max(time.elapsed_seconds().div_euclid(60.).div(2.));
}

impl EnemySpawner {
    fn get_timer(min_delay: f32, max_delay: f32) -> Timer {
        Timer::from_seconds(
            rand::thread_rng().gen_range(min_delay..max_delay),
            TimerMode::Once,
        )
    }

    fn new(min_delay: f32, max_delay: f32) -> Self {
        Self {
            min_delay: Duration::from_secs_f32(min_delay),
            max_delay: Duration::from_secs_f32(max_delay),
            timer: Self::get_timer(min_delay, max_delay),
        }
    }

    fn update(&mut self) {
        self.timer = Self::get_timer(self.min_delay.as_secs_f32(), self.max_delay.as_secs_f32());
    }

    fn get_new_spawn_location(player_position: Vec2, distance_from_player: f32) -> Vec2 {
        let new_spawn_vec = Vec2::X * distance_from_player;
        let attack_angle = rand::thread_rng().gen_range((0.)..std::f32::consts::TAU);
        let result_position = Quat::from_rotation_z(attack_angle)
            .mul_vec3(new_spawn_vec.extend(0.))
            .truncate();
        result_position + player_position
    }
}

fn spawn_enemies(
    time: Res<Time>,
    player_pos: Res<PlayerPosition>,
    mut spawner: ResMut<EnemySpawner>,
    scaling: Res<EnemyHealthScaling>,
    mut commands: Commands,
) {
    if spawner.timer.tick(time.delta()).finished() {
        let new_spawn_pos = EnemySpawner::get_new_spawn_location(player_pos.0, 400.);
        commands.spawn((
            EnemyBundle::new(Collider::circle(16.), 15. * scaling.0),
            EnemyBundle::sprite_bundle(Transform::from_xyz(new_spawn_pos.x, new_spawn_pos.y, 0.)),
            LockedAxes::ROTATION_LOCKED,
        ));
    }
}

fn update_spawner_timer(mut spawner: ResMut<EnemySpawner>) {
    if spawner.timer.finished() {
        spawner.update();
    }
}

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy: Enemy,
    rigid_body: RigidBody,
    collider: Collider,
    movement: MovementBundle,
    collision_layers: CollisionLayers,
    hurtbox: HurtboxBundle,
}

impl EnemyBundle {
    pub fn new(collider: Collider, health: f32) -> Self {
        Self {
            enemy: Enemy,
            rigid_body: RigidBody::Dynamic,
            collider,
            movement: MovementBundle::new(700., 0.9),
            collision_layers: CollisionLayers::new(
                GameLayer::Enemy,
                [GameLayer::Enemy, GameLayer::Player, GameLayer::Bullet],
            ),
            hurtbox: HurtboxBundle::new(health),
        }
    }

    pub fn sprite() -> Sprite {
        Sprite {
            color: Color::BLUE,
            custom_size: Some(Vec2::splat(32.)),
            ..Default::default()
        }
    }

    pub fn sprite_bundle(transform: Transform) -> SpriteBundle {
        SpriteBundle {
            sprite: Self::sprite(),
            transform,
            ..Default::default()
        }
    }
}

fn enemy_on_hurt_system(q_hurt: Query<Entity, (With<Hurt>, With<Enemy>)>, mut commands: Commands) {
    q_hurt.iter().for_each(|entity| {
        commands.entity(entity).remove::<Hurt>();
    });
}

fn enemy_on_dead_system(
    q_dead: Query<(Entity, &Transform), (With<Dead>, With<Enemy>)>,
    mut commands: Commands,
) {
    q_dead.iter().for_each(|(entity, transform)| {
        let enemy_translation = transform.translation;
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(5.)),
                    color: Color::WHITE,
                    ..Default::default()
                },
                transform: Transform::from_translation(enemy_translation),
                ..Default::default()
            },
            XpCrumbBundle::new(5.),
        ));
        commands.entity(entity).despawn_recursive();
    });
}

fn push_player_on_contact(
    q_player: Query<Entity, With<Player>>,
    q_enemy_collisions: Query<(&CollidingEntities, &Transform), With<Enemy>>,
    player_pos: Res<PlayerPosition>,
    mut commands: Commands,
) {
    if let Ok(player_entity) = q_player.get_single() {
        q_enemy_collisions
            .iter()
            .for_each(|(colliding_entities, enemy_tr)| {
                if colliding_entities.0.contains(&player_entity) {
                    let push_dir = player_pos.0 - enemy_tr.translation.truncate();
                    commands
                        .entity(player_entity)
                        .insert(Pushed::new(push_dir, 5.));
                }
            })
    }
}

#[derive(Component)]
pub struct Invulnerable(Timer);

impl Default for Invulnerable {
    fn default() -> Self {
        Self(Timer::from_seconds(1., TimerMode::Once))
    }
}

fn tick_invulnerable(
    time: Res<Time>,
    mut q_invulnerable: Query<(Entity, &mut Invulnerable)>,
    mut commands: Commands,
) {
    q_invulnerable
        .iter_mut()
        .for_each(|(entity, mut invulnerable)| {
            if invulnerable.0.tick(time.delta()).finished() {
                commands.entity(entity).remove::<Invulnerable>();
            }
        })
}

fn damage_player_on_contact(
    q_player: Query<Entity, (With<Player>, Without<Invulnerable>)>,
    q_enemy_collisions: Query<&CollidingEntities, With<Enemy>>,
    mut commands: Commands,
) {
    if let Ok(player_entity) = q_player.get_single() {
        q_enemy_collisions.iter().for_each(|colliding_entities| {
            if colliding_entities.0.contains(&player_entity) {
                commands.entity(player_entity).insert(DamageTaken(5.));
                commands
                    .entity(player_entity)
                    .insert(Invulnerable::default());
            }
        });
    }
}

fn move_enemies_system(
    time: Res<Time>,
    player_pos: Res<PlayerPosition>,
    mut controllers: Query<(&MovementAcceleration, &Transform, &mut LinearVelocity), With<Enemy>>,
) {
    let delta_time = time.delta_seconds_f64().adjust_precision();
    controllers
        .iter_mut()
        .for_each(|(acceleration, transform, mut velocity)| {
            let to_player_vec = (player_pos.0 - transform.translation.truncate()).normalize();
            velocity.x += to_player_vec.x * acceleration.0 * delta_time;
            velocity.y += to_player_vec.y * acceleration.0 * delta_time;
        })
}
