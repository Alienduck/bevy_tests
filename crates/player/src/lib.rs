use bevy::{
    camera_controller::pan_orbit_camera::controller::component::PanOrbitCamera, prelude::*,
};

#[derive(Resource)]
pub struct PlayerAnimations {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player);
    }
}

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let (graph, nodes) = AnimationGraph::from_clips([
        asset_server
            .load(GltfAssetLabel::Animation(0).from_asset("animations/Rig_Large_General.glb")),
        asset_server.load(
            GltfAssetLabel::Animation(1).from_asset("animations/Rig_Large_MovementAdvanced.glb"),
        ),
    ]);

    commands.insert_resource(PlayerAnimations {
        graph: graphs.add(graph),
        idle: nodes[0],
        run: nodes[1],
    });
}
