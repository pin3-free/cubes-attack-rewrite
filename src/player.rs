use crate::blink::GoInvulnerable;
use crate::enemy::Invulnerable;
use crate::prelude::*;
use bevy::ecs::system::{Command, EntityCommand, SystemState};
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

pub struct GetPushed {
    direction: Vec2,
    velocity: f32,
}

impl GetPushed {
    fn new(direction: Vec2, velocity: f32) -> Self {
        Self {
            direction,
            velocity,
        }
    }
}

impl EntityCommand for GetPushed {
    fn apply(self, id: Entity, world: &mut World) {
        let time_delta = world
            .get_resource::<Time>()
            .expect("DIO HAS ACTIVATED THE WORLD (No time found)")
            .delta_seconds();

        let mut system_state = SystemState::<Query<&mut LinearVelocity>>::new(world);
        let mut linear_velocity_query = system_state.get_mut(world);
        let mut linear_velocity = linear_velocity_query
            .get_mut(id)
            .expect("Entity has no velocity");

        let normalized_direction = self.direction.normalize();
        linear_velocity.x += time_delta * self.velocity * normalized_direction.x;
        linear_velocity.y += time_delta * self.velocity * normalized_direction.y;
    }
}

fn handle_enemy_collisions(
    mut ev_reader: EventReader<EnemyTouchedPlayerEvent>,
    mut damaged_writer: EventWriter<EntityEvent<EntityDamaged, Player>>,
    mut commands: Commands,
    q_player: Query<(Entity, &Transform), (With<Player>, Without<Enemy>)>,
    q_enemies: Query<&Transform, With<Enemy>>,
) {
    if let Ok((player_entity, player_tr)) = q_player.get_single() {
        ev_reader.read().for_each(|ev| {
            damaged_writer.send(EntityEvent::new(player_entity));

            let enemy_tr = q_enemies
                .get(ev.enemy)
                .expect("Enemy was deleted before collision could be handled");
            let push_direction = (player_tr.translation - enemy_tr.translation).truncate();
            commands
                .entity(player_entity)
                .add(GetPushed::new(push_direction, 10000.));
        });
    }
}

fn on_player_dead(mut ev_reader: EventReader<EntityEvent<EntityDead, Player>>) {
    dbg!("Player is dead and we killed him");
}

fn on_player_hit(
    mut ev_reader: EventReader<EntityEvent<EntityDamaged, Player>>,
    mut dead_writer: EventWriter<EntityEvent<EntityDead, Player>>,
    mut q_health: Query<(&mut Health, Option<&Invulnerable>), With<Player>>,
    mut commands: Commands,
) {
    let mut applied_dmg = false;
    ev_reader.read().take(1).for_each(|ev| {
        let (mut player_hp, player_invulnerable) =
            q_health.get_mut(ev.entity).expect("Player had no health");

        if player_invulnerable.is_none() {
            player_hp.cur_hp -= 5.;
            commands.entity(ev.entity).add(GoInvulnerable::new(2., 5));
            applied_dmg = true;
        }
        if player_hp.cur_hp <= 0. {
            dead_writer.send(EntityEvent::new(ev.entity));
        }
    })
}
