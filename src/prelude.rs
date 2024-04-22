use bevy::prelude::*;

pub use crate::character::{
    Action, CharacterControllerBundle, CharacterControllerPlugin, MovementAcceleration,
    MovementBundle, PlayerPosition, Pushed,
};

pub use crate::enemy::EnemyPlugin;

pub use crate::bullet::{BulletPlugin, GameLayer};

pub use crate::hurtbox::{DamageTaken, Dead, Hurt, HurtboxBundle, HurtboxPlugin};

pub use crate::xp_crumbs::{XpCrumbBundle, XpCrumbPlugin};

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;
