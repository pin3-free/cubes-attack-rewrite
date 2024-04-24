use bevy::prelude::*;

use crate::prelude::Player;

#[derive(Component)]
pub struct Hurtbox;

#[derive(Component, Clone, Copy)]
pub struct Health {
    pub cur_hp: f32,
    pub max_hp: f32,
}

impl Health {
    pub fn remaining_fraction(&self) -> f32 {
        (self.cur_hp / self.max_hp).max(0.)
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
