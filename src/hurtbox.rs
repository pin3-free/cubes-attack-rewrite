use bevy::{
    ecs::system::{EntityCommand, SystemState},
    prelude::*,
};
use std::{fmt::Debug, marker::PhantomData};

use crate::{Died, EntityEvent, Healed, TookDamage};

#[derive(Component)]
pub struct Hurtbox;

#[derive(Component, Clone, Copy)]
pub struct Health {
    pub cur_hp: f32,
    pub max_hp: f32,
}

impl Health {
    pub fn remaining_fraction(&self) -> f32 {
        (self.cur_hp / self.max_hp).clamp(0., 1.)
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.cur_hp = (self.cur_hp - amount).max(0.);
    }

    pub fn heal(&mut self, amount: f32) {
        self.cur_hp = (self.cur_hp + amount).min(self.max_hp);
    }
}

#[derive(Bundle)]
pub struct HurtboxBundle {
    hurtbox: Hurtbox,
    health: Health,
}

impl HurtboxBundle {
    pub fn new(max_health: f32) -> Self {
        Self {
            hurtbox: Hurtbox,
            health: Health {
                cur_hp: max_health,
                max_hp: max_health,
            },
        }
    }
}

pub struct TakeDamage<T: Component + Debug> {
    pub amount: f32,
    marker: PhantomData<T>,
}

impl<T: Component + Debug> TakeDamage<T> {
    pub fn new(amount: f32) -> Self {
        Self {
            amount,
            marker: Default::default(),
        }
    }
}

impl<T: Component + Debug> EntityCommand for TakeDamage<T> {
    fn apply(self, id: Entity, world: &mut World) {
        let mut system_state = SystemState::<(
            EventWriter<EntityEvent<TookDamage, T>>,
            EventWriter<EntityEvent<Died, T>>,
            Query<&mut Health, With<T>>,
        )>::new(world);

        let (mut damaged_writer, mut dead_writer, mut query) = system_state.get_mut(world);
        let mut entity_health = query.get_mut(id).expect("Entity does not have health");

        entity_health.take_damage(self.amount);
        damaged_writer.send(EntityEvent::new(id));

        if entity_health.cur_hp <= 0. {
            dead_writer.send(EntityEvent::new(id));
        }
    }
}

pub struct Heal<T: Component + Debug> {
    pub amount: f32,
    marker: PhantomData<T>,
}

impl<T: Component + Debug> Heal<T> {
    pub fn new(amount: f32) -> Self {
        Self {
            amount,
            marker: Default::default(),
        }
    }
}

impl<T: Component + Debug> EntityCommand for Heal<T> {
    fn apply(self, id: Entity, world: &mut World) {
        let mut system_state = SystemState::<(
            EventWriter<EntityEvent<Healed, T>>,
            Query<&mut Health, With<T>>,
        )>::new(world);

        let (mut healed_writer, mut query) = system_state.get_mut(world);
        let mut entity_health = query.get_mut(id).expect("Entity does not have health");

        entity_health.heal(self.amount);
        healed_writer.send(EntityEvent::new(id));
    }
}
