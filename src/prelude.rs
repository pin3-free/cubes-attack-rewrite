use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use std::fmt::Debug;
use std::marker::PhantomData;

pub use crate::character::{
    Action, CharacterControllerBundle, CharacterControllerPlugin, MovementAcceleration,
    MovementBundle, PlayerPosition, Pushed,
};

pub use crate::enemy::EnemyPlugin;

pub use crate::bullet::{BulletPlugin, GameLayer, Projectile, ProjectileHitEvent};

pub use crate::hurtbox::{DamageTaken, Dead, Health, Hurt, HurtboxBundle, HurtboxPlugin};

pub use crate::xp_crumbs::{XpCrumbBundle, XpCrumbPlugin};

#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Enemy;

pub struct RemoveEntity;

impl EntityCommand for RemoveEntity {
    fn apply(self, id: Entity, world: &mut World) {
        // dbg!("Removed", id);
        world.despawn(id);
    }
}

#[derive(Debug)]
pub struct EntityDead;

#[derive(Debug)]
pub struct EntityDamaged;

pub trait EventType: Debug {}

impl EventType for EntityDead {}
impl EventType for EntityDamaged {}

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
