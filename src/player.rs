use crate::blink::GoInvulnerable;
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
                        color: Color::GREEN,
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
    mut damaged_writer: EventWriter<EntityEvent<EntityDamaged, Player>>,
    q_player: Query<Entity, (With<Player>, Without<Enemy>)>,
) {
    if let Ok(player_entity) = q_player.get_single() {
        dbg!("Collision with enemy!");
        ev_reader.read().for_each(|_| {
            damaged_writer.send(EntityEvent::new(player_entity));
        });
    }
}

fn on_player_dead(mut ev_reader: EventReader<EntityEvent<EntityDead, Player>>) {
    dbg!("Player is dead and we killed him");
}

fn on_player_hit(
    mut ev_reader: EventReader<EntityEvent<EntityDamaged, Player>>,
    mut dead_writer: EventWriter<EntityEvent<EntityDead, Player>>,
    mut q_health: Query<&mut Health, With<Player>>,
    mut commands: Commands,
) {
    dbg!("Ouch");
    ev_reader.read().for_each(|ev| {
        let mut player_hp = q_health.get_mut(ev.entity).expect("Player had no health");

        player_hp.cur_hp -= 5.;
        commands.entity(ev.entity).add(GoInvulnerable::new(2., 5));
        if player_hp.cur_hp <= 0. {
            dead_writer.send(EntityEvent::new(ev.entity));
        }
    })
}
