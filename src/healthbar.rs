use crate::prelude::*;
use bevy::{
    ecs::{query::QueryData, system::Command},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

pub struct HealthbarPlugin;

pub struct SpawnHealthbar {
    pub tracked_entity: Entity,
}

#[derive(Component)]
pub struct TrackedEntity(pub Entity);

#[derive(Component)]
pub struct HealthbarBackground;

#[derive(Component)]
pub struct HealthbarForeground;

impl Command for SpawnHealthbar {
    fn apply(self, world: &mut World) {
        let bg_mesh = world.resource_scope(|_world, mut meshes: Mut<Assets<Mesh>>| {
            meshes.add(Mesh::from(Rectangle::new(100., 10.)))
        });

        let bg_material =
            world.resource_scope(|_world, mut materials: Mut<Assets<ColorMaterial>>| {
                materials.add(ColorMaterial::from(Color::RED))
            });

        let fg_mesh = world.resource_scope(|_world, mut meshes: Mut<Assets<Mesh>>| {
            meshes.add(Mesh::from(Rectangle::new(100., 10.)))
        });

        let fg_material =
            world.resource_scope(|_world, mut materials: Mut<Assets<ColorMaterial>>| {
                materials.add(ColorMaterial::from(Color::GREEN))
            });

        let bar = world
            .spawn((
                MaterialMesh2dBundle {
                    mesh: bg_mesh.into(),
                    material: bg_material,
                    transform: Transform::from_xyz(0., 0., 1.),
                    ..Default::default()
                },
                HealthbarBackground,
                TrackedEntity(self.tracked_entity),
            ))
            .with_children(|children| {
                children.spawn((
                    MaterialMesh2dBundle {
                        mesh: fg_mesh.into(),
                        material: fg_material,
                        transform: Transform::from_xyz(0., 0., 2.),
                        ..Default::default()
                    },
                    HealthbarForeground,
                    TrackedEntity(self.tracked_entity),
                ));
            })
            .id();

        world.entity_mut(self.tracked_entity).add_child(bar);
    }
}

fn update_bars(
    q_with_health: Query<(Entity, &Health, &Children), Changed<Health>>,
    q_backgrounds: Query<(Entity, TrackedEntity), With<HealthbarBackground>>,
    q_foregrounds: Query<(Entity, TrackedEntity), With<HealthbarForeground>>,
    mut commands: Commands,
) {
    q_with_health
        .iter()
        .for_each(|(entity, health, children)| children.iter().for_each(|bg_entity| {}));
}
