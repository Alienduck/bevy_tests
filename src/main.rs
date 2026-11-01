use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_grass::prelude::*;
use bevy_sky_gradient::prelude::*;
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PlayerPlugin,
            PhysicsPlugins::default(),
            GrassPlugins::default(),
            SkyPlugin::builder_all_features()
                .with_noise_settings(NoiseSettings {
                    noise_texture_size: 128,
                    voronoi_texture_size: 128,
                    noise_size_limit: Some(256),
                    ..default()
                })
                .set_cycle(SkyCyclePlugin {
                    sky_time_settings: SkyTimeSettings {
                        day_time_sec: 1.,
                        night_time_sec: 100.,
                        sunrise_time_sec: 10.,
                        sunset_time_sec: 10.,
                    },
                    sky_time: SkyTime {
                        time: 14.0,
                        auto_tick: true,
                    },
                })
                .build(),
        ))
        .add_plugins(PhysicsDebugPlugin)
        .insert_resource(GrassSettings {
            mid_range_density: 1.,
            instance_capacity: 300_000,
            ..GrassSettings::ULTRA
        })
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
    mut styles: ResMut<Assets<GrassStyle>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.14, 0.09),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Collider::cuboid(100.0, 0.1, 100.0),
        RigidBody::Static,
    ));

    let custom_blade = GrassBlade {
        height: Vec2::new(0.1, 1.5),
        ..GrassStyle::CRYSTAL_SPIRES.blade
    };

    commands.spawn(GrassField::new(
        Vec2::splat(100.0),
        GrassStyle::STYLIZED
            .with_density(50.0)
            .with_species(vec![GrassSpecies {
                share: 0.1,
                clustering: 1.,
                blade: custom_blade,
            }]),
        &mut styles,
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
