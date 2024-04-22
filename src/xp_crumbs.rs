use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;

use crate::prelude::GameLayer;

pub struct XpCrumbPlugin;

impl Plugin for XpCrumbPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerLevel::default())
            .add_systems(Update, (collect_xp_system, update_level_system).chain());
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
