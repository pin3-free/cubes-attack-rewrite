use bevy::prelude::*;
use std::time::Duration;

use crate::enemy::Invulnerable;
use bevy::ecs::system::EntityCommand;

pub struct BlinkPlugin;

impl Plugin for BlinkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, blink_system);
    }
}

impl EntityCommand for GoInvulnerable {
    fn apply(self, entity: Entity, world: &mut World) {
        dbg!("I'm invulnerable!");
        world
            .entity_mut(entity)
            .insert(BlinkBundle::new(self.duration, self.blink_frequency))
            .insert(Invulnerable);
    }
}

pub struct GoInvulnerable {
    pub(crate) duration: Duration,
    pub(crate) blink_frequency: u32,
}

impl Default for GoInvulnerable {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs_f32(2.),
            blink_frequency: 10,
        }
    }
}

#[allow(dead_code)]
impl GoInvulnerable {
    pub(crate) fn new(duration: f32, frequency: u32) -> Self {
        Self {
            duration: Duration::from_secs_f32(duration),
            blink_frequency: frequency,
        }
    }
}

pub struct StopInvulnerability;

impl EntityCommand for StopInvulnerability {
    fn apply(self, id: Entity, world: &mut World) {
        dbg!("Uh-oh!");
        if let Some(mut world_mut) = world.get_entity_mut(id) {
            world_mut.remove::<BlinkBundle>().remove::<Invulnerable>();
        }
    }
}

#[derive(Component)]
pub struct BlinkTimer(Timer);

#[derive(Component)]
pub struct BlinkCount(u32);

#[derive(Bundle)]
pub struct BlinkBundle {
    pub(crate) blink_timer: BlinkTimer,
    pub(crate) blinks_left: BlinkCount,
}

impl BlinkBundle {
    pub(crate) fn new(duration: Duration, frequency: u32) -> Self {
        let blink_timer = BlinkTimer(Timer::new(duration / (frequency * 2), TimerMode::Once));
        let blinks_left = BlinkCount(frequency * 2);
        Self {
            blink_timer,
            blinks_left,
        }
    }
}

pub(crate) fn blink_system(
    time: Res<Time>,
    mut q_blinking: Query<(Entity, &mut BlinkTimer, &mut BlinkCount, &mut Visibility)>,
    mut commands: Commands,
) {
    q_blinking.iter_mut().for_each(
        |(entity, mut blink_timer, mut blink_count, mut visibility)| {
            if blink_timer.0.tick(time.delta()).finished() {
                let new_visibility = match *visibility {
                    Visibility::Inherited => Visibility::Hidden,
                    Visibility::Hidden => Visibility::Visible,
                    Visibility::Visible => Visibility::Hidden,
                };
                *visibility = new_visibility;
                blink_count.0 -= 1;
                blink_timer.0.reset();

                if blink_count.0 == 0 {
                    commands.entity(entity).add(StopInvulnerability);
                    *visibility = Visibility::Visible;
                }
            }
        },
    );
}
