use std::time::Duration;

use bevy::prelude::*;
use bevy_xpbd_2d::{math::AdjustPrecision, prelude::*};
use rand::Rng;

use crate::{
    bullet::GameLayer,
    character::{MovementAcceleration, MovementBundle, PlayerPosition},
    hurtbox::{Dead, Hurt, HurtboxBundle},
    Enemy,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawner::default()).add_systems(
            Update,
            (
                enemy_on_hurt_system,
                enemy_on_dead_system,
                spawn_enemies,
                update_spawner_timer,
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
    mut commands: Commands,
) {
    if spawner.timer.tick(time.delta()).finished() {
        let new_spawn_pos = EnemySpawner::get_new_spawn_location(player_pos.0, 200.);
        commands.spawn((
            EnemyBundle::new(Collider::circle(16.)),
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
    pub fn new(collider: Collider) -> Self {
        Self {
            enemy: Enemy,
            rigid_body: RigidBody::Dynamic,
            collider,
            movement: MovementBundle::new(700., 0.9),
            collision_layers: CollisionLayers::new(
                GameLayer::Enemy,
                [GameLayer::Enemy, GameLayer::Player, GameLayer::Bullet],
            ),
            hurtbox: HurtboxBundle::new(15.),
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

fn enemy_on_dead_system(q_dead: Query<Entity, (With<Dead>, With<Enemy>)>, mut commands: Commands) {
    q_dead.iter().for_each(|entity| {
        commands.entity(entity).despawn_recursive();
    });
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
