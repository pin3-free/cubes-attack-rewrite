use crate::prelude::*;
use bevy::{
    ecs::system::{Command, SystemState},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

pub struct HealthbarPlugin;

impl Plugin for HealthbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_healthbars);
    }
}

pub struct SpawnHealthbar {
    pub tracked_entity: Entity,
}

#[derive(Component)]
pub struct HealthbarBackground;

#[derive(Component)]
pub struct HealthbarForeground;

impl SpawnHealthbar {
    fn new(tracked_entity: Entity) -> Self {
        Self { tracked_entity }
    }
}

impl Command for SpawnHealthbar {
    fn apply(self, world: &mut World) {
        let entity_health = *(SystemState::<Query<&Health>>::new(world)
            .get(world)
            .get(self.tracked_entity)
            .expect("Entity does not have health"));

        let max_width = 100.;
        let remaining_width = max_width * entity_health.remaining_fraction();
        let bar_height = 10.;

        let bg_mesh = world.resource_scope(|_world, mut meshes: Mut<Assets<Mesh>>| {
            meshes.add(Mesh::from(Rectangle::new(100., bar_height)))
        });

        let bg_material =
            world.resource_scope(|_world, mut materials: Mut<Assets<ColorMaterial>>| {
                materials.add(ColorMaterial::from(Color::RED))
            });

        let fg_mesh = world.resource_scope(|_world, mut meshes: Mut<Assets<Mesh>>| {
            meshes.add(Mesh::from(Rectangle::new(remaining_width, bar_height)))
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
            ))
            .with_children(|children| {
                children.spawn((
                    MaterialMesh2dBundle {
                        mesh: fg_mesh.into(),
                        material: fg_material,
                        transform: Transform::from_xyz(
                            -((max_width - remaining_width) / 2.),
                            0.,
                            2.,
                        ),
                        ..Default::default()
                    },
                    HealthbarForeground,
                ));
            })
            .id();

        world
            .entity_mut(self.tracked_entity)
            .insert(LinkedHealthbarId(bar));
    }
}

#[derive(Component)]
pub struct LinkedHealthbarId(Entity);

pub struct DeleteHealthbar {
    target_entity: Entity,
}

impl DeleteHealthbar {
    fn new(target_entity: Entity) -> Self {
        Self { target_entity }
    }
}

impl Command for DeleteHealthbar {
    fn apply(self, world: &mut World) {
        let linked_bar_entity = SystemState::<Query<&LinkedHealthbarId>>::new(world)
            .get(world)
            .get(self.target_entity)
            .expect("Linked bar not found")
            .0;

        world.entity_mut(linked_bar_entity).despawn_recursive();
    }
}

fn update_healthbars(
    q_with_bars: Query<Entity, (Changed<Health>, With<LinkedHealthbarId>)>,
    mut commands: Commands,
) {
    q_with_bars.iter().for_each(|entity| {
        commands.add(DeleteHealthbar::new(entity));
        commands.add(SpawnHealthbar::new(entity));
    })
}
