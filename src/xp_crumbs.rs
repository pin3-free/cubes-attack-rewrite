use std::marker::PhantomData;

use bevy::{ecs::system::Command, prelude::*};
use bevy_xpbd_2d::prelude::*;

use crate::{hurtbox::Heal, prelude::GameLayer, Player};

pub struct XpCrumbPlugin;

impl Plugin for XpCrumbPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerLevel::default()).add_systems(
            Update,
            (
                (collect_xp_system, update_level_system).chain(),
                collect_healing_system,
            ),
        );
    }
}

#[derive(Component)]
pub struct XpCrumb;

#[derive(Component)]
pub struct XpValue(f32);

#[derive(Bundle)]
pub struct XpCrumbBundle {
    xp_crumb: XpCrumb,
    rigid_body: RigidBody,
    collider: Collider,
    sensor: Sensor,
    collision_layers: CollisionLayers,
    xp_value: XpValue,
}

impl XpCrumbBundle {
    pub fn new(value: f32) -> Self {
        Self {
            xp_crumb: XpCrumb,
            rigid_body: RigidBody::Static,
            collider: Collider::rectangle(5., 5.),
            sensor: Sensor,
            collision_layers: CollisionLayers::new(GameLayer::XpCrumb, [GameLayer::Player]),
            xp_value: XpValue(value),
        }
    }
}

#[derive(Component)]
pub struct HealingCrumb;

#[derive(Component)]
pub struct HealAmount(pub f32);

#[derive(Bundle)]
pub struct HealingCrumbBundle {
    healing_crumb: HealingCrumb,
    rigid_body: RigidBody,
    collider: Collider,
    sensor: Sensor,
    collision_layers: CollisionLayers,
    heal_amount: HealAmount,
}

impl HealingCrumbBundle {
    pub fn new(heal_amount: f32) -> Self {
        Self {
            healing_crumb: HealingCrumb,
            rigid_body: RigidBody::Static,
            collider: Collider::rectangle(10., 10.),
            sensor: Sensor,
            collision_layers: CollisionLayers::new(GameLayer::HealingCrumb, [GameLayer::Player]),
            heal_amount: HealAmount(heal_amount),
        }
    }
}

pub struct SpawnCrumb<T: Component> {
    pub position: Vec2,
    marker: PhantomData<T>,
}

impl<T: Component> SpawnCrumb<T> {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            marker: Default::default(),
        }
    }
}

impl Command for SpawnCrumb<XpCrumb> {
    fn apply(self, world: &mut World) {
        world.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(5.)),
                    color: Color::WHITE,
                    ..Default::default()
                },
                transform: Transform::from_translation(self.position.extend(0.)),
                ..Default::default()
            },
            XpCrumbBundle::new(5.),
        ));
    }
}

impl Command for SpawnCrumb<HealingCrumb> {
    fn apply(self, world: &mut World) {
        world.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(10.)),
                    color: Color::GREEN,
                    ..Default::default()
                },
                transform: Transform::from_translation(self.position.extend(0.))
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
                ..Default::default()
            },
            HealingCrumbBundle::new(5.),
        ));
    }
}

#[derive(Resource)]
pub struct PlayerLevel {
    cur_level: u32,
    cur_xp: f32,
    next_level_threshold: f32,
}

impl Default for PlayerLevel {
    fn default() -> Self {
        Self {
            cur_level: 1,
            cur_xp: 0.,
            next_level_threshold: 30.,
        }
    }
}

fn update_level_system(mut player_level: ResMut<PlayerLevel>) {
    let PlayerLevel {
        cur_xp,
        next_level_threshold,
        ..
    } = *player_level;

    if cur_xp >= next_level_threshold {
        player_level.cur_level += 1;
        player_level.cur_xp = 0.;
        player_level.next_level_threshold *= 1.5;
    }
}

fn collect_xp_system(
    q_xp_collisions: Query<(Entity, &XpValue, &CollidingEntities), With<XpCrumb>>,
    mut player_level: ResMut<PlayerLevel>,
    mut commands: Commands,
) {
    q_xp_collisions
        .iter()
        .for_each(|(entity, xp_value, colliding_entities)| {
            if !colliding_entities.0.is_empty() {
                player_level.cur_xp += xp_value.0;
                commands.entity(entity).despawn_recursive();
            }
        })
}

fn collect_healing_system(
    q_healing_collisions: Query<(Entity, &HealAmount, &CollidingEntities), With<HealingCrumb>>,
    mut commands: Commands,
) {
    q_healing_collisions
        .iter()
        .for_each(|(entity, heal_amount, colliding_entities)| {
            if !colliding_entities.0.is_empty() {
                let player_entity = colliding_entities
                    .0
                    .iter()
                    .last()
                    .expect("Player is somehow missing from array");
                commands
                    .entity(*player_entity)
                    .add(Heal::<Player>::new(heal_amount.0));
                commands.entity(entity).despawn_recursive();
            }
        })
}
