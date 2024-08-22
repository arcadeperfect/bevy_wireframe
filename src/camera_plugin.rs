use bevy::{core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping}, prelude::*};

pub struct CamPlugin;

impl Plugin for CamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, (rotate_camera, move_camera));
    }
}

fn setup(mut commands: Commands) {

    let h = 1.5;

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended

            transform: Transform::from_xyz(0.0, h, 5.0).looking_at(Vec3::new(0., h, 0.), Vec3::Y),

            ..default()
        },

        BloomSettings::NATURAL,

        // BloomSettings{
        //     // intensity: 0.65,
        //     intensity: 1.0,
        //     low_frequency_boost: 0.2,
        //     low_frequency_boost_curvature: 0.95,
        //     high_pass_frequency: 1.0,
        //     prefilter_settings: BloomPrefilterSettings {
        //         threshold: 0.0,
        //         threshold_softness: 0.0,
        //     },
        //     composite_mode: BloomCompositeMode::EnergyConserving,
        // },

        OrbitCamera {
            radius: 5.0,
            speed: 1.0,
        },
        RotationEnabled(false),  // Start with rotation enabled
    ));
}


#[derive(Component)]
struct RotationEnabled(bool);



#[derive(Component)]
struct OrbitCamera {
    radius: f32,
    speed: f32,
}







fn rotate_camera(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &OrbitCamera, &mut RotationEnabled), With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Toggle rotation when 'R' is pressed
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for (_, _, mut rotation_enabled) in query.iter_mut() {
            rotation_enabled.0 = !rotation_enabled.0;
        }
    }

    for (mut transform, orbit, rotation_enabled) in query.iter_mut() {
        if rotation_enabled.0 {
            let angle = time.elapsed_seconds() * orbit.speed;
            let x = orbit.radius * angle.cos();
            let z = orbit.radius * angle.sin();

            let y = 1.5;

            transform.translation = Vec3::new(x, y, z);
            transform.look_at(Vec3::new(0., y, 0.), Vec3::Y);
        }
    }
}



// fn rotate_camera(
//     time: Res<Time>,
//     mut query: Query<(&mut Transform, &OrbitCamera), With<Camera>>,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
// ) {



//     for (mut transform, orbit) in query.iter_mut() {
//         let angle = time.elapsed_seconds() * orbit.speed;
//         let x = orbit.radius * angle.cos();
//         let z = orbit.radius * angle.sin();

//         transform.translation = Vec3::new(x, 0.0, z);
//         transform.look_at(Vec3::ZERO, Vec3::Y);
//     }
// }


// system that moves the camera with keyboard controls
fn move_camera(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    let mut direction = Vec3::ZERO;
    let move_speed = 0.01;

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        direction += Vec3::Y * move_speed;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        direction -= Vec3::Y * move_speed;
    }

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= Vec3::X * move_speed;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += Vec3::X * move_speed;
    }
    if keyboard_input.pressed(KeyCode::KeyP) {
        direction += Vec3::Z * move_speed;
    }
    if keyboard_input.pressed(KeyCode::KeyL) {
        direction -= Vec3::Z * move_speed;
    }

    if direction != Vec3::ZERO {
        for mut transform in query.iter_mut() {
            transform.translation += direction;
        }
    }
}