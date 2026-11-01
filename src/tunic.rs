use bevy::{
    material::{descriptor::RenderPipelineDescriptor, specialize::SpecializedMeshPipelineError},
    mesh::VertexAttributeValues,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{AsBindGroup, BlendState, Face, ShaderType},
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
    dispersion: f32,
    fresnel_power: f32,
    edge_sharpness: f32,
    smoke_density: f32,
    smoke_speed: f32,
    mode: f32,
    core_opacity: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
#[bind_group_data(PrismKey)]
struct PrismMaterial {
    #[uniform(0)]
    params: PrismParams,
    cull_mode: Option<Face>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct PrismKey {
    cull_mode: Option<Face>,
}

impl From<&PrismMaterial> for PrismKey {
    fn from(mat: &PrismMaterial) -> Self {
        Self {
            cull_mode: mat.cull_mode,
        }
    }
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
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;
        if let Some(target) = descriptor
            .fragment
            .as_mut()
            .and_then(|f| f.targets.get_mut(0).and_then(|t| t.as_mut()))
        {
            target.blend = Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING);
        }
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
    let pos = Vec3::new(0.0, 5., -4.0);
    let base_rot = Quat::from_euler(EulerRot::XYZ, 0.35, 0.55, -0.2);
    let mesh_handle = meshes.add(build_faceted_icosahedron(1.6));

    commands.spawn((
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(materials.add(PrismMaterial {
            params: PrismParams {
                intensity: 1.15,
                dispersion: 0.16,
                fresnel_power: 1.8,
                edge_sharpness: 0.012,
                smoke_density: 0.0,
                smoke_speed: 0.25,
                mode: 0.0,
                core_opacity: 0.88,
            },
            cull_mode: Some(Face::Front),
        })),
        Transform {
            translation: pos,
            scale: Vec3::splat(3.),
            rotation: base_rot,
        },
        Rotate::default(),
    ));

    commands.spawn((
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(materials.add(PrismMaterial {
            params: PrismParams {
                intensity: 1.65,
                dispersion: 0.13,
                fresnel_power: 2.1,
                edge_sharpness: 0.016,
                smoke_density: 0.0,
                smoke_speed: 0.25,
                mode: 0.0,
                core_opacity: 0.35,
            },
            cull_mode: Some(Face::Back),
        })),
        Transform {
            translation: pos,
            scale: Vec3::splat(3.),
            rotation: base_rot,
        },
        Rotate::default(),
    ));

    commands.spawn((
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(materials.add(PrismMaterial {
            params: PrismParams {
                intensity: 1.1,
                dispersion: 0.24,
                fresnel_power: 2.8,
                edge_sharpness: 0.01,
                smoke_density: 0.0,
                smoke_speed: 0.25,
                mode: 2.0,
                core_opacity: 0.0,
            },
            cull_mode: None,
        })),
        Transform {
            translation: pos,
            scale: Vec3::splat(3.032),
            rotation: base_rot * Quat::from_euler(EulerRot::XYZ, 0.035, -0.04, 0.025),
        },
        Rotate::default(),
    ));

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(PrismMaterial {
            params: PrismParams {
                intensity: 2.3,
                dispersion: 0.20,
                fresnel_power: 2.0,
                edge_sharpness: 0.02,
                smoke_density: 30.4,
                smoke_speed: 0.85,
                mode: 1.0,
                core_opacity: 0.0,
            },
            cull_mode: Some(Face::Back),
        })),
        Transform {
            translation: pos,
            scale: Vec3::splat(3.1),
            rotation: base_rot,
        },
        Rotate::default(),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.75, 0.3, 1.0),
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
    }
}
