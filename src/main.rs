use line_material::{
    generate_edge_line_list_data, LineMaterial,
};
use mesh_ops::random_color_mesh;
use std::time::Duration;
use bevy::{
    animation::animate_targets, prelude::*, render::{
        mesh::VertexAttributeValues, render_asset::RenderAssetUsages,
    }
};
use fill_material::FillMaterial;
use outline_material::OutlineMaterial;

mod camera_plugin;
mod fill_material;
mod line_material;
mod mesh_ops;
mod outline_material;

const PATH: &str = "astro/scene.gltf";
// const PATH: &str = "sphere_flat.gltf";

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(camera_plugin::CamPlugin)
        .add_plugins(MaterialPlugin::<FillMaterial>::default())
        .add_plugins(MaterialPlugin::<OutlineMaterial>::default())
        .add_plugins(MaterialPlugin::<LineMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, play_animation_once_loaded.before(animate_targets))
        .add_systems(Update, mesh_added)
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
        transform: Transform::from_xyz(0.0, 0.3, 0.0),
        ..default()
    });
}

// Once the scene is loaded, start the animation
fn play_animation_once_loaded(
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


fn mesh_added(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, query: Query<(Entity, &Handle<Mesh>), Added<Handle<Mesh>>>) {
    for (entity, mesh_handle) in query.iter() {
        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            mesh_to_wireframe(mesh);
        }
    }
}




fn mesh_to_wireframe(mesh: &mut Mesh) {

    random_color_mesh(mesh);

    let lines = generate_edge_line_list_data(mesh);

    let mut line_mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD);

    let positions: Vec<[f32; 3]> = lines
                .lines
                .iter()
                .flat_map(|(start, end)| vec![start.position, end.position])
                .collect();

                let colors: Vec<[f32; 4]> = lines
                .lines
                .iter()
                .flat_map(|(start, end)| vec![start.color, end.color])
                .flatten()
                .collect();

            let normal: Vec<[f32; 3]> = lines
                .lines
                .iter()
                .flat_map(|(start, end)| vec![start.normal, end.normal])
                .flatten()
                .collect();

            let joint_indices: Vec<[u16; 4]> = lines
                .lines
                .iter()
                .flat_map(|(start, end)| vec![start.joint_indices, end.joint_indices])
                .flatten()
                .collect();

            let joint_weights: Vec<[f32; 4]> = lines
                .lines
                .iter()
                .flat_map(|(start, end)| vec![start.joint_weights, end.joint_weights])
                .flatten()
                .collect();

            let joint_indices_count = joint_indices.len();
            let joint_weights_count = joint_weights.len();

            println!("{} {}", joint_indices_count, joint_weights_count);

            assert_eq!(
                joint_indices_count, joint_weights_count,
                "Joint indices and weights must have the same length"
            );

            line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            line_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            line_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normal);
            line_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_INDEX,VertexAttributeValues::Uint16x4(joint_indices));
            line_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weights);

    *mesh = line_mesh;
 
}
