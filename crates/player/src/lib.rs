use std::time::Duration;

use bevy::{prelude::*, world_serialization::WorldInstanceReady};

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Player {
    pub speed: f32,
    pub anim_state: AnimState,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum AnimState {
    Idle,
    Walk,
    Run,
}

#[derive(Resource)]
pub struct PlayerAnimations {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(camera::SmartCameraPlugin)
            .add_systems(Startup, setup_player)
            .add_systems(Update, (move_player, update_animations).chain());
    }
}

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let (graph, nodes) = AnimationGraph::from_clips([
        asset_server
            .load(GltfAssetLabel::Animation(0).from_asset("animations/Rig_Large_General.glb")),
        asset_server.load(
            GltfAssetLabel::Animation(1).from_asset("animations/Rig_Large_MovementBasic.glb"),
        ),
        asset_server.load(
            GltfAssetLabel::Animation(2).from_asset("animations/Rig_Large_MovementAdvanced.glb"),
        ),
    ]);

    commands.insert_resource(PlayerAnimations {
        graph: graphs.add(graph),
        idle: nodes[0],
        walk: nodes[1],
        run: nodes[2],
    });

    let player = commands
        .spawn((
            Player {
                speed: 4.0,
                anim_state: AnimState::Idle,
            },
            WorldAssetRoot(
                asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("characters/Mannequin_Large.glb")),
            ),
        ))
        .id();

    commands.spawn((
        Camera3d::default(),
        camera::SmartCamera {
            camera_subject: Some(player),
            distance: 10.0,
            mode: camera::CameraMode::ThirdPerson,
            ..default()
        },
    ));

    commands.entity(player).observe(init_character_animations);
}

fn init_character_animations(
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

fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    cam_q: Query<&Transform, (With<camera::SmartCamera>, Without<Player>)>,
    mut player_q: Query<(&mut Transform, &mut Player)>,
) {
    let Ok(cam_tf) = cam_q.single() else { return };
    let Ok((mut player_tf, mut player)) = player_q.single_mut() else {
        return;
    };

    let mut input = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        input.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        input.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        input.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        input.x += 1.0;
    }

    let Some(dir_2d) = input.try_normalize() else {
        player.anim_state = AnimState::Idle;
        return;
    };

    let forward = Vec3::new(cam_tf.forward().x, 0.0, cam_tf.forward().z).normalize_or_zero();
    let right = Vec3::new(cam_tf.right().x, 0.0, cam_tf.right().z).normalize_or_zero();
    let move_dir = (forward * dir_2d.y + right * dir_2d.x).normalize_or_zero();

    let sprinting = keys.pressed(KeyCode::ShiftLeft);
    let current_speed = if sprinting {
        player.speed * 2.0
    } else {
        player.speed
    };
    player.anim_state = if sprinting {
        AnimState::Run
    } else {
        AnimState::Walk
    };

    player_tf.translation += move_dir * current_speed * time.delta_secs();

    if move_dir.length_squared() > 0.001 {
        let target_rot = Quat::from_rotation_arc(Vec3::NEG_Z, move_dir);
        player_tf.rotation = player_tf
            .rotation
            .slerp(target_rot, 12.0 * time.delta_secs());
    }
}

fn update_animations(
    anims: Res<PlayerAnimations>,
    player_q: Query<(&Player, Entity), Changed<Player>>,
    children: Query<&Children>,
    mut anim_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (player, root_entity) in &player_q {
        let target_node = match player.anim_state {
            AnimState::Idle => anims.idle,
            AnimState::Walk => anims.walk,
            AnimState::Run => anims.run,
        };

        for child in children.iter_descendants(root_entity) {
            if let Ok((mut anim_player, mut transitions)) = anim_players.get_mut(child) {
                if !anim_player.is_playing_animation(target_node) {
                    transitions
                        .play(&mut anim_player, target_node, Duration::from_millis(200))
                        .repeat();
                }
            }
        }
    }
}
