use bevy::{
    animation::animate_targets, core::FrameCount, ecs::query, gltf::GltfPlugin, pbr::wireframe::Wireframe, prelude::*, render::{
        mesh::{skinning::SkinnedMesh, MeshVertexAttribute, VertexAttributeValues},
        render_asset::RenderAssetUsages, render_resource::VertexFormat,
    }, scene::{SceneInstance, SceneInstanceReady}
};
use fill_material::FillMaterial;
use line_material::{generate_edge_line_list_data, LineMaterial};
use mesh_ops::random_color_mesh;
use outline_material::OutlineMaterial;
use std::time::Duration;

mod camera_plugin;
mod fill_material;
mod line_material;
mod mesh_ops;
mod outline_material;

// const PATH: &str = "astro/scene.gltf";
const PATH: &str = "astro_custom/scene.gltf";
// const PATH: &str = "sphere_flat.gltf";
// const PATH: &str = "sphere.gltf";

#[derive(Resource)]
struct MyScene(Entity);

#[derive(Component)]
struct WireFrameScene;

#[derive(Component)]
struct OriginalSceneTag;

const ATTRIBUTE_CUSTOM: MeshVertexAttribute =
    MeshVertexAttribute::new("Custom", 2137464976, VertexFormat::Float32);

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
        })
        .add_plugins(
            DefaultPlugins.set(
                GltfPlugin::default()
                    // Map a custom glTF attribute name to a `MeshVertexAttribute`.
                    .add_custom_vertex_attribute("CUSTOM", ATTRIBUTE_CUSTOM),
            ),
        )        .add_plugins(camera_plugin::CamPlugin)
        .add_plugins(MaterialPlugin::<FillMaterial>::default())
        .add_plugins(MaterialPlugin::<OutlineMaterial>::default())
        .add_plugins(MaterialPlugin::<LineMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, play_animation_once_loaded.before(animate_targets))
        .add_systems(Update, system)
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
    assets: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let mut graph = AnimationGraph::new();
    let animations = graph
        .add_clips(
            [GltfAssetLabel::Animation(0).from_asset(PATH)]
                .into_iter()
                .map(|path| assets.load(path)),
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

    let newScene = SceneBundle {
        scene: assets.load(GltfAssetLabel::Scene(0).from_asset(PATH)),
        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(0.0)),
        ..default()
    };

    let entity = commands.spawn((newScene, OriginalSceneTag)).id();

    commands.insert_resource(MyScene(entity));
}

fn system(
    mut commands: Commands,
    mut events: EventReader<SceneInstanceReady>,
    my_scene_entity: Res<MyScene>,
    children: Query<&Children>,
    meshes: Query<(Entity, &Handle<Mesh>)>,
    skinned_meshes: Query<&SkinnedMesh>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    mut fill_materials: ResMut<Assets<FillMaterial>>, // Add FillMaterial resource
    mut outline_materials: ResMut<Assets<OutlineMaterial>>, // Add FillMaterial resource
) {
    for event in events.read() {
        if event.parent == my_scene_entity.0 {
            for entity in children.iter_descendants(event.parent) {
                if let Ok((entity, mesh_handle)) = meshes.get(entity) {
                    if let Some(original_mesh) = mesh_assets.get(mesh_handle) {
                        commands.entity(entity).remove::<Handle<StandardMaterial>>();

                        // Add FillMaterial component
                        let fill_material_handle = fill_materials.add(FillMaterial::default());
                        commands.entity(entity).insert(fill_material_handle.clone());

                        // Clone the mesh
                        let mut new_mesh = original_mesh.clone();
                        mesh_to_wireframe(&mut new_mesh);

                        // Add FillMaterial component
                        let outline_material_handle =
                            outline_materials.add(OutlineMaterial::default());
                        commands
                            .entity(entity)
                            .insert(outline_material_handle.clone());

                        // Clone the mesh
                        let mut new_mesh = original_mesh.clone();
                        mesh_to_wireframe(&mut new_mesh);

                        // Add the new mesh to the assets and get a handle to it
                        let new_mesh_handle = mesh_assets.add(new_mesh);

                        // Get the SkinnedMesh component if it exists
                        let skinned_mesh = skinned_meshes.get(entity).cloned();

                        // Prepare the bundle
                        let mut bundle = MaterialMeshBundle {
                            mesh: new_mesh_handle,
                            material: line_materials.add(LineMaterial::default()),
                            ..Default::default()
                        };

                        // Spawn the new entity
                        let mut entity_commands = commands.spawn(bundle);

                        // If the original entity had a SkinnedMesh component, add it to the new entity
                        if let Ok(skinned_mesh) = skinned_mesh {
                            entity_commands.insert(skinned_mesh);
                        }
                    }
                }
            }
        }
    }
}

fn mesh_to_wireframe(mesh: &mut Mesh) {
    // let mut mesh = m.clone();

    random_color_mesh(mesh);

    let lines = generate_edge_line_list_data(&mesh);

    let mut line_mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::LineList,
        RenderAssetUsages::RENDER_WORLD,
    );

    let positions: Vec<[f32; 3]> = lines
        .lines
        .iter()
        .flat_map(|(start, end)| vec![start.position, end.position])
        .collect();

    line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    let colors: Vec<[f32; 4]> = lines
        .lines
        .iter()
        .flat_map(|(start, end)| vec![start.color, end.color])
        .flatten()
        .collect();

    line_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    let normal: Vec<[f32; 3]> = lines
        .lines
        .iter()
        .flat_map(|(start, end)| vec![start.normal, end.normal])
        .collect();

    line_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normal);

    if let Some(VertexAttributeValues::Uint16x4(_)) = mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX) {
        let joint_indices: Vec<[u16; 4]> = lines
            .lines
            .iter()
            .flat_map(|(start, end)| vec![start.joint_indices, end.joint_indices])
            .flatten()
            .collect();
        line_mesh.insert_attribute(
            Mesh::ATTRIBUTE_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(joint_indices),
        );
    }

    if let Some(VertexAttributeValues::Float32x4(_)) = mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
    {
        let joint_weights: Vec<[f32; 4]> = lines
            .lines
            .iter()
            .flat_map(|(start, end)| vec![start.joint_weights, end.joint_weights])
            .flatten()
            .collect();
        line_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weights);
    }

    *mesh = line_mesh;
}

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

// fn system(
//     mut commands: Commands,
//     assets: Res<AssetServer>,

//     mut events: EventReader<SceneInstanceReady>,
//     my_scene_entity: Res<MyScene>,
//     children: Query<&Children>,
//     scene_instances: Query<&SceneInstance>,
//     scenes: Query<(Entity, &Handle<Scene>)>,
//     // mut meshes: Query<&mut Mesh>,
//     mut meshes: Query<(Entity, &Handle<Mesh>)>,
//     mut mesh_assets: ResMut<Assets<Mesh>>,
//     mut scene_spawner: ResMut<SceneSpawner>,

// ) {
//     for event in events.read() {
//         if event.parent == my_scene_entity.0 {
//             if let Ok(scene_instance) = scene_instances.get(event.parent) {
//                 println!("Found scene instance");

//                 if let Ok((entity, scene_handle)) = scenes.get(event.parent) {
//                     let cloned_scene_handle = scene_handle.clone();

//                     let new_scene_bundle = SceneBundle {
//                         scene: scene_handle.clone(),
//                         transform: Transform::from_xyz(1.0, 0.0, 0.0),
//                         ..default()
//                     };

//                     let new_entity = commands.spawn(new_scene_bundle).id();
//                 }
//             }

//             for entity in children.iter_descendants(event.parent) {
//                 if let Ok((entity, mesh_handle)) = meshes.get(entity) {
//                     if let Some(mesh) = mesh_assets.get_mut(mesh_handle) {
//                         mesh_to_wireframe(mesh);
//                     }
//                 }
//             }
//         }
//     }
// }
