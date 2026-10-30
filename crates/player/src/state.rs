use bevy::{animation::AnimationPlayer, prelude::*, world_serialization::WorldInstanceReady};
use std::time::Duration;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
            .add_systems(Update, update_animations)
            .add_systems(PostUpdate, check_animation_completion);
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
    player_q: Query<(&crate::Player, Entity), Changed<crate::Player>>,
    mut anim_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    anims: Res<PlayerAnimations>,
    children: Query<&Children>,
) {
    for (player, root_entity) in &player_q {
        let (target_node, looped, speed) = match player.anim_state {
            AnimState::Idle => (anims.idle, true, 1.0),
            AnimState::Walk => (anims.walk, true, 1.0),
            AnimState::Run => (anims.run, true, 1.2),
            AnimState::StartJump => (anims.start_jump, false, 1.0),
            AnimState::IdleJump => (anims.idle_jump, true, 1.0),
            AnimState::Landing => (anims.landing, false, 1.5),
        };

        for child in children.iter_descendants(root_entity) {
            if let Ok((mut anim_player, mut transitions)) = anim_players.get_mut(child) {
                if !anim_player.is_playing_animation(target_node) {
                    let active_anim =
                        transitions.play(&mut anim_player, target_node, Duration::from_millis(200));

                    active_anim.set_speed(speed);
                    if looped {
                        active_anim.repeat();
                    } else {
                        active_anim.set_repeat(bevy::animation::RepeatAnimation::Never);
                    }
                }
            }
        }
    }
}

fn check_animation_completion(
    mut player_q: Query<(&mut crate::Player, Entity)>,
    anim_players: Query<&AnimationPlayer>,
    anims: Res<PlayerAnimations>,
    children: Query<&Children>,
) {
    for (mut player, root_entity) in &mut player_q {
        let target_node_to_check = match player.anim_state {
            AnimState::StartJump => Some(anims.start_jump),
            AnimState::Landing => Some(anims.landing),
            _ => None,
        };

        if let Some(node_to_check) = target_node_to_check {
            for child in children.iter_descendants(root_entity) {
                if let Ok(anim_player) = anim_players.get(child) {
                    if let Some(anim) = anim_player.animation(node_to_check) {
                        if anim.is_finished() {
                            player.anim_state = match player.anim_state {
                                AnimState::StartJump => AnimState::IdleJump,
                                AnimState::Landing => AnimState::Idle,
                                _ => player.anim_state,
                            };
                        }
                    }
                }
            }
        }
    }
}
