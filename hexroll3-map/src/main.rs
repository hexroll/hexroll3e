mod hexmap;

use core::f32;

use bevy::{
    prelude::*,
    render::{
        camera::ScalingMode,
        view::{ColorGrading, ColorGradingSection, RenderLayers},
    },
};

use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use hexmap::{HexMap, MainCamera};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgba(0.0, 0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_plugins(HexMap)
        .run();
}

fn setup(mut commands: Commands, mut ambient_light: ResMut<AmbientLight>) {
    ambient_light.brightness = 0.0;
    commands.spawn((
        MainCamera,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 1.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        PanOrbitCamera {
            radius: Some(1000.0),
            yaw: Some(0.0),
            pitch_lower_limit: Some(f32::consts::PI),
            pitch_upper_limit: Some(f32::consts::PI),
            yaw_lower_limit: Some(0.0),
            yaw_upper_limit: Some(0.0),
            button_pan: MouseButton::Left,
            button_orbit: MouseButton::Right,
            ..default()
        },
        Camera {
            hdr: true,
            order: 0,
            ..default()
        },
        RenderLayers::from_layers(&[0]),
        ColorGrading {
            shadows: ColorGradingSection {
                contrast: 0.98,
                ..default()
            },
            ..default()
        },
    ));
}
