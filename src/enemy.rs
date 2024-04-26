use std::{ops::Div, time::Duration};

use bevy::ecs::system::{Command, SystemState};
use bevy::prelude::*;
use bevy_xpbd_2d::{math::AdjustPrecision, prelude::*};
use rand::Rng;

use crate::healthbar::{DeleteHealthbar, SpawnHealthbar};
use crate::{
    bullet::{Projectile, ProjectileDamage},
    hurtbox::TakeDamage,
    prelude::*,
    xp_crumbs::{HealingCrumb, SpawnCrumb, XpCrumb},
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyHealthScaling(1.))
            .add_event::<EnemyTouchedPlayerEvent>()
            .add_event::<EntityEvent<TookDamage, Enemy>>()
            .add_event::<EntityEvent<Died, Enemy>>()
            .insert_resource(EnemySpawner::default())
            .add_systems(
                Update,
                (
                    (handle_projectile_hits).run_if(on_event::<ProjectileHitEvent<Enemy>>()),
                    (enemy_on_dead_system).run_if(on_event::<EntityEvent<Died, Enemy>>()),
                    spawn_enemies,
                    update_enemy_health_scaling,
                    update_spawner_timer,
                    emit_player_contact_events,
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
}

struct SpawnEnemy {
    position: Vec2,
}

impl SpawnEnemy {
    fn angle_from_player(
        player_position: Vec2,
        distance_from_player: f32,
        attack_angle: f32,
    ) -> Self {
        let new_spawn_vec = Vec2::X * distance_from_player;
        let result_position = Quat::from_rotation_z(attack_angle)
            .mul_vec3(new_spawn_vec.extend(0.))
            .truncate();

        Self {
            position: result_position + player_position,
        }
    }

    fn random_angle(player_position: Vec2, distance_from_player: f32) -> Self {
        let attack_angle = rand::thread_rng().gen_range((0.)..(std::f32::consts::TAU));
        Self::angle_from_player(player_position, distance_from_player, attack_angle)
    }
}

impl Command for SpawnEnemy {
    fn apply(self, world: &mut World) {
        let scaling = world
            .get_resource::<EnemyHealthScaling>()
            .expect("Failed to obtain enemy health scaling handle");

        let enemy = world
            .spawn((
                EnemyBundle::new(Collider::circle(16.), 15. * scaling.0),
                EnemyBundle::sprite_bundle(Transform::from_xyz(
                    self.position.x,
                    self.position.y,
                    0.,
                )),
                LockedAxes::ROTATION_LOCKED,
            ))
            .id();

        let mut system_state = SystemState::<Commands>::new(world);
        let mut commands = system_state.get_mut(world);
        commands.add(SpawnHealthbar::new(enemy));
        system_state.apply(world);
    }
}

fn spawn_enemies(
    time: Res<Time>,
    player_pos: Res<PlayerPosition>,
    mut spawner: ResMut<EnemySpawner>,
    mut commands: Commands,
) {
    if spawner.timer.tick(time.delta()).finished() {
        commands.add(SpawnEnemy::random_angle(player_pos.0, 400.));
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

fn handle_projectile_hits(
    q_projectiles: Query<&ProjectileDamage, With<Projectile>>,
    mut ev_reader: EventReader<ProjectileHitEvent<Enemy>>,
    mut commands: Commands,
) {
    ev_reader.read().for_each(
        |ProjectileHitEvent::<Enemy> {
             projectile, target, ..
         }| {
            let projectile_damage = q_projectiles
                .get(*projectile)
                .expect("Failed to find projectile");

            commands
                .entity(*target)
                .add(TakeDamage::<Enemy>::new(projectile_damage.0));
            commands.entity(*projectile).add(RemoveEntity);
        },
    );
}

fn enemy_on_dead_system(
    q_enemies: Query<&Transform, With<Enemy>>,
    mut dead_reader: EventReader<EntityEvent<Died, Enemy>>,
    mut commands: Commands,
) {
    dead_reader
        .read()
        .for_each(|EntityEvent::<Died, Enemy> { entity, .. }| {
            let enemy_translation = q_enemies
                .get(*entity)
                .expect("Entity not found")
                .translation;

            match rand::thread_rng().gen_range(0..100) {
                0..=90 => commands.add(SpawnCrumb::<XpCrumb>::new(enemy_translation.truncate())),
                _ => commands.add(SpawnCrumb::<HealingCrumb>::new(
                    enemy_translation.truncate(),
                )),
            };

            commands.entity(*entity).add(RemoveEntity);
        });
}

#[derive(Component)]
pub struct Invulnerable;

#[derive(Event)]
pub struct EnemyTouchedPlayerEvent {
    pub enemy: Entity,
}

impl EnemyTouchedPlayerEvent {
    fn new(enemy: Entity) -> Self {
        Self { enemy }
    }
}

fn emit_player_contact_events(
    q_player: Query<Entity, With<Player>>,
    q_enemies: Query<(Entity, &CollidingEntities), With<Enemy>>,
    mut ev_writer: EventWriter<EnemyTouchedPlayerEvent>,
) {
    if let Ok(player_entity) = q_player.get_single() {
        q_enemies
            .iter()
            .for_each(|(enemy_entity, colliding_entities)| {
                if colliding_entities.0.contains(&player_entity) {
                    ev_writer.send(EnemyTouchedPlayerEvent::new(enemy_entity));
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
