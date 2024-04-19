use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;
use bullet::{BulletPlugin, GameLayer};
use character::{Action, CharacterControllerBundle, CharacterControllerPlugin};
use enemy::{EnemyBundle, EnemyPlugin};
use hurtbox::{HurtboxBundle, HurtboxPlugin};

mod bullet;
mod character;
mod enemy;
mod hurtbox;

#[derive(Component)]
pub struct MainCamera;

fn setup(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::splat(32.)),
                ..default()
            },
            ..Default::default()
        },
        CharacterControllerBundle::new(Collider::circle(16.), Action::default_input_map()),
        Player,
        HurtboxBundle::new(15.),
        CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy]),
        LockedAxes::ROTATION_LOCKED,
    ));
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            CharacterControllerPlugin,
            BulletPlugin,
            EnemyPlugin,
            HurtboxPlugin,
        ))
        .add_systems(Startup, setup)
        .insert_resource(Gravity(Vec2::ZERO))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;
