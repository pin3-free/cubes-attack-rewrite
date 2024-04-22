use crate::blink::Blink;
use crate::enemy::Invulnerable;
use crate::prelude::*;
use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;

#[derive(Component, Debug)]
pub struct Player;

pub struct SpawnPlayer {
    position: Vec2,
    size: f32,
    health: f32,
    collection_radius: f32,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EntityEvent<EntityDead, Player>>()
            .add_event::<EntityEvent<EntityDamaged, Player>>()
            .add_systems(
                Update,
                (handle_enemy_collisions,).run_if(on_event::<EnemyTouchedPlayerEvent>()),
            )
            .add_systems(
                Update,
                (on_player_dead,).run_if(on_event::<EntityEvent<EntityDead, Player>>()),
            )
            .add_systems(
                Update,
                (on_player_hit,).run_if(on_event::<EntityEvent<EntityDamaged, Player>>()),
            );
    }
}

impl Default for SpawnPlayer {
    fn default() -> Self {
        Self {
            size: 32.,
            health: 15.,
            collection_radius: 200.,
            position: Default::default(),
        }
    }
}

impl Command for SpawnPlayer {
    fn apply(self, world: &mut World) {
        world
            .spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::RED,
                        custom_size: Some(Vec2::splat(self.size)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(self.position.extend(0.)),
                    ..Default::default()
                },
                CharacterControllerBundle::new(
                    Collider::circle(self.size / 2.),
                    Action::default_input_map(),
                ),
                Player,
                HurtboxBundle::new(self.health),
                CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy]),
                LockedAxes::ROTATION_LOCKED,
            ))
            .with_children(|children| {
                children.spawn((
                    Collider::circle(self.collection_radius),
                    Sensor,
                    CollisionLayers::new(GameLayer::Player, [GameLayer::XpCrumb]),
                ));
            });
    }
}

fn handle_enemy_collisions(
    mut ev_reader: EventReader<EnemyTouchedPlayerEvent>,
    mut dead_writer: EventWriter<EntityEvent<EntityDead, Player>>,
    mut damaged_writer: EventWriter<EntityEvent<EntityDamaged, Player>>,
    mut q_player: Query<(Entity, &mut Health), (With<Player>, Without<Enemy>)>,
) {
    if let Ok((player_entity, mut health)) = q_player.get_single_mut() {
        ev_reader.read().for_each(|_| {
            health.cur_hp -= 5.;
            damaged_writer.send(EntityEvent::new(player_entity));

            if health.cur_hp <= 0. {
                dbg!();
                dead_writer.send(EntityEvent::new(player_entity));
            }
        })
    }
}

fn on_player_dead(mut ev_reader: EventReader<EntityEvent<EntityDead, Player>>) {
    dbg!("Player is dead and we killed him");
}

fn on_player_hit(
    mut ev_reader: EventReader<EntityEvent<EntityDamaged, Player>>,
    mut commands: Commands,
) {
    ev_reader.read().for_each(|ev| {
        let inv_duration = 1.;
        commands.entity(ev.entity).add(Blink::new(inv_duration, 10));
        commands
            .entity(ev.entity)
            .insert(Invulnerable::new(inv_duration));
    })
}
