use crate::prelude::*;
use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;
use blink::BlinkPlugin;
use healthbar::SpawnHealthbar;

mod blink;
mod bullet;
mod character;
mod enemy;
mod healthbar;
mod hurtbox;
mod player;
mod prelude;
mod xp_crumbs;

fn setup(mut commands: Commands) {
    commands.add(SpawnPlayer::default());
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn spawn_player_hotbar(mut commands: Commands, q_player: Query<Entity, With<Player>>) {
    let player_entity = q_player.get_single().expect("Player not found");

    commands.add(SpawnHealthbar {
        tracked_entity: player_entity,
    });
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
            XpCrumbPlugin,
            PlayerPlugin,
            BlinkPlugin,
            HealthbarPlugin,
        ))
        .add_systems(Startup, (setup, spawn_player_hotbar).chain())
        .insert_resource(Gravity(Vec2::ZERO))
        .run();
}
