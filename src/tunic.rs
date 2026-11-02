use bevy::{
    material::{descriptor::RenderPipelineDescriptor, specialize::SpecializedMeshPipelineError},
    mesh::VertexAttributeValues,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};

pub struct TunicPlugin;

impl Plugin for TunicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<PrismMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, rotate);
    }
}

#[derive(ShaderType, Clone, Copy)]
struct PrismParams {
    intensity: f32,
    edge_thickness_px: f32,
    smear_length: f32,
    dispersion: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct PrismMaterial {
    #[uniform(0)]
    params: PrismParams,
}

impl Material for PrismMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tunic_prism.wesl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(Component)]
struct Rotate {
    speed: f32,
}

impl Default for Rotate {
    fn default() -> Self {
        Self { speed: 0.5 }
    }
}

fn build_faceted_icosahedron(radius: f32) -> Mesh {
    let mut mesh = Sphere::new(radius).mesh().ico(0).unwrap();
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    if let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        let len = pos.len();
        let mut barycentrics = Vec::with_capacity(len);
        for _ in 0..(len / 3) {
            barycentrics.push([1.0, 0.0, 0.0, 1.0]);
            barycentrics.push([0.0, 1.0, 0.0, 1.0]);
            barycentrics.push([0.0, 0.0, 1.0, 1.0]);
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, barycentrics);
    }
    mesh
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PrismMaterial>>,
) {
    let pos = Vec3::new(0.0, 5.5, -8.0);

    commands.spawn((
        Mesh3d(meshes.add(build_faceted_icosahedron(1.6))),
        MeshMaterial3d(materials.add(PrismMaterial {
            params: PrismParams {
                intensity: 1.75,
                edge_thickness_px: 1.6,
                smear_length: 1.28,
                dispersion: 1.,
            },
        })),
        Transform {
            translation: pos,
            rotation: Quat::from_euler(EulerRot::XYZ, 0.35, 0.55, -0.2),
            scale: Vec3::splat(3.),
            ..default()
        },
        Rotate::default(),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.8, 0.35, 1.0),
            intensity: 100_000.0,
            range: 18.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_translation(pos),
    ));
}

fn rotate(prism_q: Query<(&mut Transform, &Rotate)>, time: Res<Time>) {
    for (mut transform, rotation) in prism_q {
        transform.rotate_x(1. * rotation.speed * time.delta_secs());
        transform.rotate_z(0.44 * rotation.speed * time.delta_secs());
    }
}
