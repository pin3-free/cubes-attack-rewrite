use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use std::fmt::Debug;
use std::marker::PhantomData;

pub use crate::character::{
    Action, CharacterControllerBundle, CharacterControllerPlugin, MovementAcceleration,
    MovementBundle, PlayerPosition,
};

pub use crate::enemy::{EnemyPlugin, EnemyTouchedPlayerEvent};

pub use crate::bullet::{BulletPlugin, GameLayer, ProjectileHitEvent};

pub use crate::hurtbox::{Health, HurtboxBundle};

pub use crate::xp_crumbs::{XpCrumbBundle, XpCrumbPlugin};

pub use crate::player::{Player, PlayerPlugin, SpawnPlayer};

pub use crate::healthbar::HealthbarPlugin;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Debug)]
pub struct Enemy;

pub struct RemoveEntity;

impl EntityCommand for RemoveEntity {
    fn apply(self, id: Entity, world: &mut World) {
        // dbg!("Removed", id);
        world.entity_mut(id).despawn_recursive();
    }
}

#[derive(Debug)]
pub struct Died;

#[derive(Debug)]
pub struct TookDamage;

#[derive(Debug)]
pub struct Healed;

pub trait EventType: Debug {}

impl EventType for Died {}
impl EventType for TookDamage {}
impl EventType for Healed {}

#[derive(Event)]
pub struct EntityEvent<EvT: EventType, EnT: Component + Debug> {
    pub entity: Entity,
    entity_type: PhantomData<EvT>,
    event_type: PhantomData<EnT>,
}

impl<EvT: EventType, EnT: Component + Debug> EntityEvent<EvT, EnT> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            entity_type: Default::default(),
            event_type: Default::default(),
        }
    }
}
