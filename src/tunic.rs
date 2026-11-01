use bevy::{
    material::{descriptor::RenderPipelineDescriptor, specialize::SpecializedMeshPipelineError},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{AsBindGroup, Face, ShaderType},
    shader::ShaderRef,
};

pub struct TunicPlugin;

impl Plugin for TunicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<PrismMaterial>::default())
            .add_systems(Startup, setup);
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
    _pad: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
#[bind_group_data(PrismKey)]
struct PrismMaterial {
    #[uniform(0)]
    params: PrismParams,
    is_smoke: bool,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct PrismKey {
    is_smoke: bool,
}

impl From<&PrismMaterial> for PrismKey {
    fn from(mat: &PrismMaterial) -> Self {
        Self {
            is_smoke: mat.is_smoke,
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
        descriptor.primitive.cull_mode = if key.bind_group_data.is_smoke {
            Some(Face::Back)
        } else {
            None
        };
        Ok(())
    }
}

fn build_faceted_icosahedron(radius: f32) -> Mesh {
    let mut mesh = Sphere::new(radius).mesh().ico(1).unwrap();
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    if let Some(pos) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        let len = pos.len();
        let mut uvs = Vec::with_capacity(len);
        for _ in 0..(len / 3) {
            uvs.push([0.0, 0.0]);
            uvs.push([1.0, 0.0]);
            uvs.push([0.5, 1.0]);
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }
    mesh
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PrismMaterial>>,
) {
    let pos = Vec3::new(0.0, 2.5, -4.0);
    let base_rot = Quat::from_euler(EulerRot::XYZ, 0.4, 0.6, 0.1);
    let mesh_handle = meshes.add(build_faceted_icosahedron(1.6));

    for (scale, rot, intensity, dispersion) in [
        (Vec3::ONE, base_rot, 1.5, 0.08),
        (
            Vec3::splat(1.035),
            Quat::from_euler(EulerRot::XYZ, 0.44, 0.57, 0.13),
            0.9,
            0.15,
        ),
    ] {
        commands.spawn((
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(materials.add(PrismMaterial {
                params: PrismParams {
                    intensity,
                    dispersion,
                    fresnel_power: 2.2,
                    edge_sharpness: 14.0,
                    smoke_density: 0.0,
                    smoke_speed: 0.3,
                    mode: 0.0,
                    _pad: 0.0,
                },
                is_smoke: false,
            })),
            Transform {
                translation: pos,
                scale,
                rotation: rot,
            },
        ));
    }

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(PrismMaterial {
            params: PrismParams {
                intensity: 1.,
                dispersion: 0.5,
                fresnel_power: 2.0,
                edge_sharpness: 1.0,
                smoke_density: 3.2,
                smoke_speed: 0.45,
                mode: 1.0,
                _pad: 0.0,
            },
            is_smoke: true,
        })),
        Transform {
            translation: pos,
            scale: Vec3::splat(1.38),
            rotation: base_rot,
        },
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.85, 0.45, 1.0),
            intensity: 120_000.0,
            range: 18.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_translation(pos),
    ));
}
