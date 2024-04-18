use bevy::prelude::*;

pub struct HurtboxPlugin;

impl Plugin for HurtboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, take_damage_system);
    }
}

#[derive(Component)]
pub struct Hurtbox;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct DamageTaken(pub f32);

#[derive(Component)]
pub struct Hurt;

#[derive(Component)]
pub struct Dead;

#[derive(Bundle)]
pub struct HurtboxBundle {
    hurtbox: Hurtbox,
    health: Health,
}

impl HurtboxBundle {
    pub fn new(health: f32) -> Self {
        Self {
            hurtbox: Hurtbox,
            health: Health(health),
        }
    }
}

fn take_damage_system(
    mut query: Query<(&mut Health, &DamageTaken, Entity)>,
    mut commands: Commands,
) {
    query.iter_mut().for_each(|(mut health, damage, entity)| {
        health.0 -= damage.0;
        commands.entity(entity).insert(Hurt);

        if health.0 <= 0. {
            commands.entity(entity).insert(Dead);
        }
        commands.entity(entity).remove::<DamageTaken>();
    })
}
