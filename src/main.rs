use avian3d::prelude::*;
use bevy::prelude::*;
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PlayerPlugin, PhysicsPlugins::default()))
        .add_plugins(PhysicsDebugPlugin)
        .insert_gizmo_config(
            PhysicsGizmos {
                aabb_color: Some(bevy::color::palettes::css::GREEN.into()),
                ..default()
            },
            GizmoConfig::default(),
        )
        .add_systems(Startup, setup_scene)
        .add_systems(Update, toggle_debug_render_collision)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
        Collider::cuboid(100.0, 0.1, 100.0),
        RigidBody::Static,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn toggle_debug_render_collision(
    mut gizmo_store_config: ResMut<GizmoConfigStore>,
    input: ResMut<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyT) {
        let (config, _) = gizmo_store_config.config_mut::<avian3d::prelude::PhysicsGizmos>();
        config.enabled = !config.enabled;
    }
}
