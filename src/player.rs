use crate::blink::GoInvulnerable;
use crate::enemy::Invulnerable;
use crate::hurtbox::TakeDamage;
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
        app.add_event::<EntityEvent<Died, Player>>()
            .add_event::<EntityEvent<TookDamage, Player>>()
            .add_event::<EntityEvent<Healed, Player>>()
            .add_systems(
                Update,
                (handle_enemy_collisions,).run_if(on_event::<EnemyTouchedPlayerEvent>()),
            )
            .add_systems(
                Update,
                (on_player_dead,).run_if(on_event::<EntityEvent<Died, Player>>()),
            )
            .add_systems(
                Update,
                (on_player_heal,).run_if(on_event::<EntityEvent<Healed, Player>>()),
            )
            .add_systems(
                Update,
                (on_player_hit,).run_if(on_event::<EntityEvent<TookDamage, Player>>()),
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
                CollisionLayers::new(
                    GameLayer::Player,
                    [GameLayer::Enemy, GameLayer::HealingCrumb],
                ),
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
    mut commands: Commands,
    q_player: Query<(Entity, &Transform, Option<&Invulnerable>), (With<Player>, Without<Enemy>)>,
    q_enemies: Query<&Transform, With<Enemy>>,
) {
    if let Ok((player_entity, player_tr, player_invulnerable)) = q_player.get_single() {
        let mut applied_dmg = false;
        ev_reader.read().for_each(|ev| {
            if !applied_dmg && !player_invulnerable.is_some() {
                commands
                    .entity(player_entity)
                    .add(TakeDamage::<Player>::new(5.))
                    .add(GoInvulnerable::new(2., 5));
                applied_dmg = true;
            }

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

fn on_player_dead(mut ev_reader: EventReader<EntityEvent<Died, Player>>) {
    dbg!("Player is dead and we killed him");
}

fn on_player_hit(mut ev_reader: EventReader<EntityEvent<TookDamage, Player>>) {
    dbg!("Skill issue");
}

fn on_player_heal(mut ev_reader: EventReader<EntityEvent<Healed, Player>>) {
    dbg!("Skill solution");
}
