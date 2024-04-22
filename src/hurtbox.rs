use bevy::prelude::*;

use crate::prelude::Player;

pub struct HurtboxPlugin;

impl Plugin for HurtboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                take_damage_system,
                spawn_player_hp_system,
                remove_player_hp_system,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
pub struct Hurtbox;

#[derive(Component)]
pub struct Health {
    pub cur_hp: f32,
    pub max_hp: f32,
}

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

#[derive(Component)]
pub struct TrackedEntity(Entity);

#[derive(Bundle)]
pub struct HealthBarBundle {
    entity_tracked: TrackedEntity,
    ui: HealthBarUiBundle,
}

impl HealthBarBundle {
    fn new(entity_tracked: Entity, width: Val, height: Val) -> Self {
        Self {
            entity_tracked: TrackedEntity(entity_tracked),
            ui: HealthBarUiBundle::new(width, height),
        }
    }

    fn adjust(mut self, health: &Health) -> Self {
        self.ui.foreground.style.width = Val::Percent(100. * (health.cur_hp / health.max_hp));
        self.ui.text.text.sections[0].value = format!("{}/{}", health.cur_hp, health.max_hp);
        self
    }
}

#[derive(Bundle)]
pub struct HealthBarUiBundle {
    background: NodeBundle,
    foreground: NodeBundle,
    text: TextBundle,
}

impl HealthBarUiBundle {
    fn new(width: Val, height: Val) -> Self {
        Self {
            background: NodeBundle {
                style: Style {
                    width,
                    height,
                    display: Display::Flex,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    position_type: PositionType::Relative,
                    ..Default::default()
                },
                background_color: Color::BLACK.into(),
                ..Default::default()
            },
            foreground: NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width,
                    height,
                    left: Val::Percent(0.),
                    ..Default::default()
                },
                background_color: Color::GREEN.into(),
                ..Default::default()
            },
            text: TextBundle::from_section(
                "100%",
                TextStyle {
                    font_size: 15.,
                    color: Color::WHITE,
                    ..Default::default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Relative,
                ..Default::default()
            }),
        }
    }

    fn build(self, commands: &mut Commands) -> Entity {
        commands
            .spawn((self.background, HealthBarBackground))
            .with_children(|children| {
                children.spawn((self.foreground, HealthBarForeground));
                children.spawn((self.text, HealthBarText));
            })
            .id()
    }
}

impl Default for HealthBarUiBundle {
    fn default() -> Self {
        Self::new(Val::Px(200.), Val::Px(20.))
    }
}

#[derive(Component)]
pub struct HealthBarBackground;

#[derive(Component)]
pub struct HealthBarForeground;

#[derive(Component)]
pub struct HealthBarText;

fn spawn_player_hp_system(
    mut commands: Commands,
    q_player: Query<(Entity, &Health), With<Player>>,
) {
    if let Ok((player_entity, player_hp)) = q_player.get_single() {
        let hbar =
            HealthBarBundle::new(player_entity, Val::Px(200.), Val::Px(20.)).adjust(player_hp);
        hbar.ui.build(&mut commands);
    } else {
        dbg!("FUCK! The player isn't spawned in yet");
    }
}

fn remove_player_hp_system(
    mut commands: Commands,
    q_entity: Query<Entity, With<HealthBarBackground>>,
) {
    q_entity
        .iter()
        .for_each(|entity| commands.entity(entity).despawn_recursive());
}

fn take_damage_system(
    mut query: Query<(&mut Health, &DamageTaken, Entity)>,
    mut commands: Commands,
) {
    query.iter_mut().for_each(|(mut health, damage, entity)| {
        health.cur_hp -= damage.0;
        commands.entity(entity).insert(Hurt);
        if health.cur_hp <= 0. {
            commands.entity(entity).insert(Dead);
        }
        commands.entity(entity).remove::<DamageTaken>();
    })
}
