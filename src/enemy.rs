use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;

use crate::{
    bullet::GameLayer,
    character::MovementBundle,
    hurtbox::{Dead, Hurt, HurtboxBundle},
    Enemy,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (enemy_on_hurt_system, enemy_on_dead_system).chain());
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
}

fn enemy_on_hurt_system(q_hurt: Query<Entity, (With<Hurt>, With<Enemy>)>, mut commands: Commands) {
    q_hurt.iter().for_each(|entity| {
        println!("Enemy is hurt!");
        commands.entity(entity).remove::<Hurt>();
    });
}

fn enemy_on_dead_system(q_dead: Query<Entity, (With<Dead>, With<Enemy>)>, mut commands: Commands) {
    q_dead.iter().for_each(|entity| {
        println!("Enemy is dead!");
        commands.entity(entity).despawn_recursive();
    });
}
