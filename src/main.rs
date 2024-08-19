use mesh_ops::{random_color_mesh, smooth_normals};
use std::time::Duration;

use bevy::{animation::animate_targets, prelude::*};
use fill_material::FillMaterial;
use outline_material::OutlineMaterial;

mod camera_plugin;
mod fill_material;
mod mesh_ops;
mod outline_material;

const PATH: &str = "astro/scene.gltf";

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(camera_plugin::cam_plugin)
        .add_plugins(MaterialPlugin::<FillMaterial>::default())
        .add_plugins(MaterialPlugin::<OutlineMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, setup_scene_once_loaded.before(animate_targets))
        .add_systems(Update, process_scene)
        .run();
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    graph: Handle<AnimationGraph>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let mut graph = AnimationGraph::new();
    let animations = graph
        .add_clips(
            [
                // GltfAssetLabel::Animation(2).from_asset(PATH),
                // GltfAssetLabel::Animation(1).from_asset(PATH),
                GltfAssetLabel::Animation(0).from_asset(PATH),
            ]
            .into_iter()
            .map(|path| asset_server.load(path)),
            1.0,
            graph.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let graph = graphs.add(graph);
    commands.insert_resource(Animations {
        animations,
        graph: graph.clone(),
    });

    // Character
    commands.spawn(SceneBundle {
        scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(PATH)),
        ..default()
    });
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        println!("animation player added");
        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

fn process_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut fill_materials: ResMut<Assets<FillMaterial>>,
    mut outline_materials: ResMut<Assets<OutlineMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    query: Query<
        (Entity, &Handle<StandardMaterial>, &Handle<Mesh>),
        Added<Handle<Mesh>>,
    >,
) {
    for (entity, _material_handle, mesh_handle) in query.iter() {
        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            random_color_mesh(mesh);
            smooth_normals(mesh);

            // Create new materials
            let fill_material = fill_materials.add(FillMaterial {
                color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                displacement: 10.0,
            });

            let outline_material = outline_materials.add(OutlineMaterial {
                flat_color: Vec4::new(0.0, 1.0, 1.0, 1.0),
                outline_width: 2.1,
                ..default()
            });

            let standard_material = standard_materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 1.0, 0.0),
                ..default()
            });

            // Remove the StandardMaterial component
            commands.entity(entity).remove::<Handle<StandardMaterial>>();

            // Add the new material components to the original mesh
            commands
                .entity(entity)
                // .insert(standard_material)
                .insert(fill_material)
                .insert(outline_material)
            ;
        }
    }
}
