use std::f32::consts::FRAC_PI_2;

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

/// Different camera modes
/// TODO: add SecondPerson (follow)
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum CameraMode {
    FirstPerson,
    #[default]
    ThirdPerson,
}

/// Enum to take inputs from keyboard and mouse
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraInput {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

/// The smart camera is based on the roblox's camera
#[derive(Component)]
pub struct SmartCamera {
    /// The entity which the camera will follow, else it will be disable
    pub camera_subject: Option<Entity>,
    /// The camera mode, see [`CameraMode`]
    pub mode: CameraMode,
    /// Distance between camera and followed entity (when CameraMode is on ThirdPerson)
    pub distance: f32,
    /// Min Camera distance
    pub min_distance: f32,
    /// Max Camera distance
    pub max_distance: f32,
    /// If this property is true, the player can scroll to zoom in or out with the camera to control the distance
    pub can_zoom: bool,
    /// The sensitivity of the scrolling mouse behavior
    pub scroll_sensitivity: f32,
    /// The mouse motion sensitivity
    pub sensitivity: f32,
    /// Vertical angle (up/down)
    pub pitch: f32,
    /// Horizontal angle (left/right)
    pub yaw: f32,
    /// The field of view of the camera
    pub fov: f32,
    /// Is the mouse lock behavior active ?
    pub mouse_lock_enabled: bool,
    /// When Shift or desire input pressed, the mouse is locked
    pub mouse_lock: bool,
    /// Custom input for mouse lock
    pub custom_mouse_lock_input: Option<CameraInput>,
    /// When set to true, the motion will be apply only if right click or custom input is press
    pub motion_on_input: bool,
    /// The custom input for *motion_on_input*
    pub custom_input: Option<CameraInput>,
    /// When camera in first person, apply on offset to go into the head
    pub first_person_offset: Vec3,
}

impl Default for SmartCamera {
    fn default() -> Self {
        Self {
            camera_subject: None,
            mode: CameraMode::ThirdPerson,
            distance: 5.0,
            min_distance: 1.0,
            max_distance: 10.0,
            can_zoom: false,
            sensitivity: 1.0,
            scroll_sensitivity: 0.25,
            pitch: 0.0,
            yaw: 0.0,
            fov: 70.0,
            mouse_lock_enabled: false,
            mouse_lock: false,
            custom_mouse_lock_input: None,
            motion_on_input: false,
            custom_input: None,
            first_person_offset: Vec3::new(0.0, 1.5, 0.0),
        }
    }
}
pub struct SmartCameraPlugin;

impl Plugin for SmartCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, update_smart_camera)
            .add_systems(
                Update,
                (camera_mouse_control, zoom_handler, lock_mouse).chain(),
            );
    }
}

fn update_smart_camera(
    mut camera_query: Query<(&SmartCamera, &mut Transform, &mut Projection)>,
    transforms_query: Query<&Transform, Without<SmartCamera>>,
) {
    let Ok((smart_camera, mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };
    if let Projection::Perspective(ref mut perspective) = *projection {
        if perspective.fov != smart_camera.fov {
            perspective.fov = smart_camera.fov;
        }
    }
    if let Some(target_entity) = smart_camera.camera_subject {
        if let Ok(target_transform) = transforms_query.get(target_entity) {
            let rotation =
                Quat::from_euler(EulerRot::YXZ, smart_camera.yaw, smart_camera.pitch, 0.0);
            match smart_camera.mode {
                CameraMode::ThirdPerson => {
                    let offset = rotation * Vec3::new(0.0, 0.0, smart_camera.distance);
                    camera_transform.translation =
                        target_transform.translation + Vec3::new(0.0, 1.0, 0.0) + offset;
                }
                CameraMode::FirstPerson => {
                    camera_transform.translation =
                        target_transform.translation + smart_camera.first_person_offset;
                }
            }
            camera_transform.rotation = rotation;
        }
    }
}

fn camera_mouse_control(
    mut motion_evr: MessageReader<MouseMotion>,
    mut query: Query<&mut SmartCamera>,
    mut cursor: Single<&mut CursorOptions>,
    mouse_inputs: Res<ButtonInput<MouseButton>>,
    key_inputs: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut smart_camera) = query.single_mut() else {
        return;
    };

    cursor.visible = !smart_camera.mouse_lock;

    if smart_camera.motion_on_input {
        let is_pressed = match smart_camera.custom_input {
            Some(CameraInput::Keyboard(keycode)) => key_inputs.pressed(keycode),
            Some(CameraInput::Mouse(mouse_button)) => mouse_inputs.pressed(mouse_button),
            None => mouse_inputs.pressed(MouseButton::Right),
        };

        if !is_pressed {
            motion_evr.clear();
            return;
        }
    }

    let mut mouse_delta = Vec2::ZERO;
    for msg in motion_evr.read() {
        mouse_delta += msg.delta;
    }

    if mouse_delta != Vec2::ZERO
        && (smart_camera.mouse_lock || mouse_inputs.pressed(MouseButton::Right))
    {
        let sensitivity = smart_camera.sensitivity * 0.01;
        smart_camera.yaw -= mouse_delta.x * sensitivity;
        smart_camera.pitch -= mouse_delta.y * sensitivity;
        smart_camera.pitch = smart_camera
            .pitch
            .clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
    }
}

fn lock_mouse(
    mut camera_query: Query<&mut SmartCamera>,
    mut cursor: Single<&mut CursorOptions>,
    key_inputs: Res<ButtonInput<KeyCode>>,
    mouse_inputs: Res<ButtonInput<MouseButton>>,
) {
    let Ok(mut camera) = camera_query.single_mut() else {
        return;
    };

    if !camera.mouse_lock_enabled {
        return;
    }

    let is_pressed = match camera.custom_mouse_lock_input {
        Some(CameraInput::Keyboard(keycode)) => key_inputs.just_pressed(keycode),
        Some(CameraInput::Mouse(mouse_button)) => mouse_inputs.just_pressed(mouse_button),
        None => key_inputs.just_pressed(KeyCode::ShiftLeft),
    };

    if is_pressed {
        if camera.mouse_lock {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        } else {
            cursor.visible = false;
            cursor.grab_mode = CursorGrabMode::Locked;
        }
        camera.mouse_lock = !camera.mouse_lock;
    }
}

fn zoom_handler(mut camera_q: Query<&mut SmartCamera>, mut mouse_wheel: MessageReader<MouseWheel>) {
    let Ok(mut camera) = camera_q.single_mut() else {
        return;
    };

    for wheel in mouse_wheel.read() {
        camera.distance = (camera.distance + (wheel.y * -1.0) * camera.scroll_sensitivity)
            .clamp(camera.min_distance, camera.max_distance);
    }
}

/// Take a position in the map.\
/// Return a Vec3 where X and Y are the screen position of the given position and Z is the distance between camera and the given position.\
/// Return also a bool which is true when the givben position is on screen
pub fn world_to_screen_point(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    target: Vec3,
) -> (Vec3, bool) {
    let distance = camera_transform.translation().distance(target);

    match camera.world_to_viewport(camera_transform, target) {
        Ok(screen_pos) => {
            let mut is_on_screen = false;

            if let Some(viewport_size) = camera.logical_viewport_size() {
                is_on_screen = screen_pos.x >= 0.0
                    && screen_pos.x <= viewport_size.x
                    && screen_pos.y >= 0.0
                    && screen_pos.y <= viewport_size.y;
            }

            (
                Vec3::new(screen_pos.x, screen_pos.y, distance),
                is_on_screen,
            )
        }
        Err(_) => (Vec3::new(0.0, 0.0, distance), false),
    }
}
