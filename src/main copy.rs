use bevy::prelude::*;
use bevy::{app::Startup, DefaultPlugins};

mod camera_plugin;

const PATH: &str = "animated_astro/scene.gltf";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(camera_plugin::cam_plugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SceneBundle {
        scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(PATH)),
        ..default()
    });
}
