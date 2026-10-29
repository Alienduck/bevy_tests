use std::time::Duration;

use bevy::{prelude::*, world_serialization::WorldInstanceReady};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum AnimState {
    Idle,
    Walk,
    Run,
    StartJump,
    IdleJump,
    Landing,
}

#[derive(Resource)]
pub struct PlayerAnimations {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub start_jump: AnimationNodeIndex,
    pub idle_jump: AnimationNodeIndex,
    pub landing: AnimationNodeIndex,
}

pub struct PlayerStatePlugin;

impl Plugin for PlayerStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, update_animations);
    }
}

fn setup(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
) {
    let (graph, nodes) = AnimationGraph::from_clips([
        asset_server.load(
            GltfAssetLabel::Animation(6).from_asset("animations/Rig_Medium/Rig_Medium_General.glb"),
        ),
        asset_server.load(
            GltfAssetLabel::Animation(9)
                .from_asset("animations/Rig_Medium/Rig_Medium_MovementBasic.glb"),
        ),
        asset_server.load(
            GltfAssetLabel::Animation(6)
                .from_asset("animations/Rig_Medium/Rig_Medium_MovementBasic.glb"),
        ),
        asset_server.load(
            GltfAssetLabel::Animation(5)
                .from_asset("animations/Rig_Medium/Rig_Medium_MovementBasic.glb"),
        ),
        asset_server.load(
            GltfAssetLabel::Animation(3)
                .from_asset("animations/Rig_Medium/Rig_Medium_MovementBasic.glb"),
        ),
        asset_server.load(
            GltfAssetLabel::Animation(4)
                .from_asset("animations/Rig_Medium/Rig_Medium_MovementBasic.glb"),
        ),
    ]);

    commands.insert_resource(PlayerAnimations {
        graph: graphs.add(graph),
        idle: nodes[0],
        walk: nodes[1],
        run: nodes[2],
        start_jump: nodes[3],
        idle_jump: nodes[4],
        landing: nodes[5],
    });
}

pub fn init_character_animations(
    trigger: On<WorldInstanceReady>,
    anims: Res<PlayerAnimations>,
    children: Query<&Children>,
    mut players: Query<(Entity, &mut AnimationPlayer)>,
    mut commands: Commands,
) {
    for child in children.iter_descendants(trigger.event_target()) {
        if let Ok((entity, mut player)) = players.get_mut(child) {
            let mut transitions = AnimationTransitions::new();
            transitions
                .play(&mut player, anims.idle, Duration::ZERO)
                .repeat();
            commands
                .entity(entity)
                .insert((transitions, AnimationGraphHandle(anims.graph.clone())));
        }
    }
}

fn update_animations(
    mut player_q: Query<(&mut crate::Player, Entity), Changed<crate::Player>>,
    mut anim_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    anims: Res<PlayerAnimations>,
    children: Query<&Children>,
) {
    for (mut player, root_entity) in &mut player_q {
        let target_node = match player.anim_state {
            AnimState::Idle => anims.idle,
            AnimState::Walk => anims.walk,
            AnimState::Run => anims.run,
            AnimState::StartJump => anims.start_jump,
            AnimState::IdleJump => anims.idle_jump,
            AnimState::Landing => anims.landing,
        };

        for child in children.iter_descendants(root_entity) {
            if let Ok((mut anim_player, mut transitions)) = anim_players.get_mut(child) {
                if !anim_player.is_playing_animation(target_node) {
                    transitions
                        .play(&mut anim_player, target_node, Duration::from_millis(200))
                        .repeat();
                }
                if let Some(anim) = anim_player.animation(anims.start_jump) {
                    if anim.just_completed() {
                        player.anim_state = AnimState::IdleJump;
                    }
                }
                if let Some(anim) = anim_player.animation(anims.landing) {
                    if anim.just_completed() {
                        player.anim_state = AnimState::Idle;
                    }
                }
            }
        }
    }
}
