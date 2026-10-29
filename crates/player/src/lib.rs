use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_grass::prelude::*;
use bevy_sky_gradient::prelude::*;

use crate::state::AnimState;

mod state;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Player {
    pub speed: f32,
    pub jump_force: f32,
    pub can_jump: bool,
    pub jump_recover_dist: f32,
    pub anim_state: AnimState,
}

#[derive(Component)]
pub struct CustomPlayerTexture {
    pub texture: Handle<Image>,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((camera::SmartCameraPlugin, state::PlayerStatePlugin))
            .add_systems(Startup, (setup_player, debug_gltf_animations).chain())
            .add_systems(PostUpdate, toggle_first_person)
            .add_systems(
                Update,
                (apply_player_texture_system, move_player, jump_handler, jump).chain(),
            );
    }
}

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("textures/mannequin_texture.png");
    const CAPSULE_RADIUS: f32 = 0.4;
    const CAPSULE_LENGTH: f32 = 1.2;
    const CAPSULE_HALF_HEIGHT: f32 = CAPSULE_LENGTH * 0.5 + CAPSULE_RADIUS;

    let player = commands
        .spawn((
            Player {
                speed: 5.0,
                jump_force: 4.,
                can_jump: true,
                jump_recover_dist: 0.02,
                anim_state: AnimState::Idle,
            },
            WorldAssetRoot(asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("animations/Rig_Medium/Rig_Medium_General.glb"),
            )),
            CustomPlayerTexture {
                texture: texture_handle,
            },
            RigidBody::Dynamic,
            Collider::compound(vec![(
                Vec3::new(0.0, CAPSULE_HALF_HEIGHT, 0.0),
                Quat::IDENTITY,
                Collider::capsule(CAPSULE_RADIUS, CAPSULE_LENGTH),
            )]),
            ShapeCaster::new(
                Collider::capsule(CAPSULE_RADIUS, CAPSULE_LENGTH),
                Vec3::new(0.0, CAPSULE_HALF_HEIGHT, 0.0),
                Quat::IDENTITY,
                Dir3::NEG_Y,
            ),
            LockedAxes::ROTATION_LOCKED,
            GrassInteractorFromCollider::default(),
        ))
        .observe(state::init_character_animations)
        .id();

    commands.spawn((
        Camera3d::default(),
        camera::SmartCamera {
            camera_subject: Some(player),
            distance: 10.0,
            mode: camera::CameraMode::ThirdPerson,
            mouse_lock_enabled: true,
            mouse_lock: true,
            custom_mouse_lock_input: Some(camera::CameraInput::Keyboard(KeyCode::Escape)),
            can_zoom: true,
            max_distance: 30.0,
            ..default()
        },
        SkyboxMagnetTag,
    ));
}

fn jump(
    mut player_linear_q: Query<(&mut LinearVelocity, &mut Player)>,
    input: ResMut<ButtonInput<KeyCode>>,
) {
    let Ok((mut linear, mut player)) = player_linear_q.single_mut() else {
        return;
    };
    if input.just_pressed(KeyCode::Space) && player.can_jump {
        player.can_jump = false;
        linear.y += player.jump_force;
        player.anim_state = AnimState::StartJump;
    }
}

fn jump_handler(mut player_q: Query<(&mut Player, &ShapeCaster, &ShapeHits)>) {
    let Ok((mut player, _, hits)) = player_q.single_mut() else {
        return;
    };
    if player.can_jump {
        return;
    }
    for hit in hits.iter() {
        if hit.distance < player.jump_recover_dist {
            player.can_jump = true;
            player.anim_state = AnimState::Landing;
        }
    }
}

fn apply_player_texture_system(
    mut commands: Commands,
    q_new: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        Added<MeshMaterial3d<StandardMaterial>>,
    >,
    q_parent: Query<&ChildOf>,
    q_custom: Query<&CustomPlayerTexture>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mat_handle) in &q_new {
        let mut current = entity;
        let mut texture: Option<Handle<Image>> = None;
        loop {
            if let Ok(custom) = q_custom.get(current) {
                texture = Some(custom.texture.clone());
                break;
            }
            match q_parent.get(current) {
                Ok(child_of) => current = child_of.parent(),
                Err(_) => break,
            }
        }
        let Some(texture) = texture else { continue };

        let Some(src) = materials.get(&mat_handle.0) else {
            continue;
        };
        let mut copy = src.clone();
        copy.base_color = Color::WHITE;
        copy.base_color_texture = Some(texture);

        let new_handle = materials.add(copy);
        commands.entity(entity).insert(MeshMaterial3d(new_handle));
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
    if player.can_jump {
        player.anim_state = if sprinting {
            AnimState::Run
        } else {
            AnimState::Walk
        };
    }

    player_tf.translation += move_dir * current_speed * time.delta_secs();

    if move_dir.length_squared() > 0.001 {
        let target_rot = Quat::from_rotation_arc(Vec3::Z, move_dir);
        player_tf.rotation = player_tf
            .rotation
            .slerp(target_rot, 12.0 * time.delta_secs());
    }
}

fn toggle_first_person(
    mut player_q: Query<&mut Visibility, With<Player>>,
    mut camera_q: Query<&mut camera::SmartCamera>,
    inputs: ResMut<ButtonInput<KeyCode>>,
) {
    let Ok(mut player_visibility) = player_q.single_mut() else {
        return;
    };
    if inputs.just_pressed(KeyCode::KeyQ) {
        if let Ok(mut camera) = camera_q.single_mut() {
            camera.mode = match camera.mode {
                camera::CameraMode::FirstPerson => {
                    *player_visibility = Visibility::Visible;
                    camera::CameraMode::ThirdPerson
                }
                camera::CameraMode::ThirdPerson => {
                    *player_visibility = Visibility::Hidden;
                    camera::CameraMode::FirstPerson
                }
            };
        };
    }
}

fn debug_gltf_animations(gltf_assets: Res<Assets<Gltf>>) {
    for (_, gltf) in gltf_assets.iter() {
        for (name, _) in &gltf.named_animations {
            info!("Animation disponible : {}", name);
        }
    }
}
