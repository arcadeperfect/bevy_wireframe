use line_material::{
    generate_edge_line_list_data, generate_edge_line_list_indices, IndexLineList, IntEdge,
    LineMaterial,
};
use mesh_ops::{random_color_mesh, smooth_normals};
use std::time::Duration;

use bevy::{
    animation::animate_targets, ecs::query, prelude::*, render::{
        mesh::VertexAttributeValues, render_asset::RenderAssetUsages,
        render_resource::VertexFormat, renderer::RenderDevice,
    }, scene::SceneInstanceReady, text::scale_value, utils::HashSet
};
use fill_material::FillMaterial;
use outline_material::OutlineMaterial;

use bevy::prelude::*;
use bevy::render::render_resource::{Buffer, BufferInitDescriptor, BufferUsages};

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
        // .add_systems(Update, play_animation_once_loaded.before(animate_targets))
        .add_systems(Update, process_scene)
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
    // // Build the animation graph
    // let mut graph = AnimationGraph::new();
    // let animations = graph
    //     .add_clips(
    //         [
    //             // GltfAssetLabel::Animation(2).from_asset(PATH),
    //             // GltfAssetLabel::Animation(1).from_asset(PATH),
    //             GltfAssetLabel::Animation(0).from_asset(PATH),
    //         ]
    //         .into_iter()
    //         .map(|path| asset_server.load(path)),
    //         1.0,
    //         graph.root,
    //     )
    //     .collect();

    // // Insert a resource with the current scene information
    // let graph = graphs.add(graph);
    // commands.insert_resource(Animations {
    //     animations,
    //     graph: graph.clone(),
    // });

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


fn process_scene(
    mut commands: Commands,
    mut ready_events: EventReader<SceneInstanceReady>,
    query: Query<(&Children, Option<&Handle<Mesh>>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for scene in ready_events.read() {
        println!("a");
        let root_entity = scene.parent;
        process_entity_recursive(root_entity, &query, &mut meshes);
    }
}

fn process_entity_recursive(
    entity: Entity,
    query: &Query<(&Children, Option<&Handle<Mesh>>)>,
    meshes: &mut Assets<Mesh>,
) {
    // println!("Processing entity: {:?}", entity);

    if let Ok((children, mesh_handle)) = query.get(entity) {
        // println!("Entity has {} children", children.len());
        
        match mesh_handle {
            Some(_) => println!("Entity has a mesh handle"),
            None => (),
            // None => println!("Entity does not have a mesh handle"),
        }

        // If this entity has a mesh, modify it
        if let Some(mesh_handle) = mesh_handle {
            println!("Attempting to get mesh from handle");
            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                println!("Successfully retrieved mesh, modifying...");
                mesh_to_wireframe(mesh);
            } else {
                println!("Failed to retrieve mesh from handle");
            }
        }

        // Recursively process children
        for &child in children.iter() {
            process_entity_recursive(child, query, meshes);
        }
    } else {
        // println!("Failed to get components for entity: {:?}", entity);
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

// fn process_scene(
//     mut commands: Commands,
//     mut scene_spawner: ResMut<SceneSpawner>,
//     scene_handles: Res<Assets<Scene>>,
//     // query: Query<(Entity, &Handle<Scene>)>,
//     // query: Query<(Entity, &Handle<Scene>), Added<Handle<Mesh>>>,
//     query: Query<
//         (
//             Entity,
//             &Handle<Scene>,
//             // &Handle<StandardMaterial>,
//             // &Handle<Mesh>,
//             // &GlobalTransform,
//         ),
//         // Added<Handle<Mesh>>,
//     >,
// ) {

    
//     for (entity, scene_handle  ) in query.iter() {
//         println!("a");
//         if let Some(scene) = scene_handles.get(scene_handle){
//             println!("b");
//         }
//     }

//     // for (entity, scene_handle) in query.iter() {
//     //     println!("a");
//     //     if let Some(scene) = scene_handles.get(scene_handle) {
//     //         println!("b");
//     //         println!("entity: {:?}, scene: {:?}", entity, scene_handle);

//     //         // // Iterate through the entities and components of the scene
//     //         // for entity in &scene.entities {
//     //         //     println!("Entity in scene: {:?}", entity);

//     //         //     // If you want to modify or add components, you can use the entity from the scene
//     //         //     // Example: commands.entity(*entity).insert(SomeComponent { ... });
//     //         // }
//     //     }

//     //     // Optionally, despawn the original entity if it's no longer needed
//     //     // commands.entity(entity).despawn();
//     // }
// }

// fn process_scene(
//     // mut world: World,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut line_materials: ResMut<Assets<LineMaterial>>,
//     mut standard_materials: ResMut<Assets<StandardMaterial>>,
//     query: Query<
//         (
//             Entity,
//             &Handle<StandardMaterial>,
//             &Handle<Mesh>,
//             &GlobalTransform,
//             &Handle<Scene>
//         ),
//         Added<Handle<Mesh>>,
//     >,
// ) {
//     for (entity, _material_handle, mesh_handle, global_transform, scene) in query.iter() {
//         if let Some(mesh) = meshes.get_mut(mesh_handle) {

//             println!("1");

//             let line_material = line_materials.add(LineMaterial {});

//             random_color_mesh(mesh);

//             // VertexAttributeValues::Float32x3(Mesh::ATTRIBUTE_POSITION);

//             // let index_line_list: IndexLineList = generate_edge_line_list_indices(mesh);

//             let lines = generate_edge_line_list_data(mesh);

//             println!("{} lines", lines.lines.len());

//             let mut line_mesh = Mesh::new(
//                 bevy::render::render_resource::PrimitiveTopology::LineList,
//                 RenderAssetUsages::RENDER_WORLD,
//             );

//             let positions: Vec<[f32; 3]> = lines
//                 .lines
//                 .iter()
//                 .flat_map(|(start, end)| vec![start.position, end.position])
//                 .collect();

//             let colors: Vec<[f32; 4]> = lines
//                 .lines
//                 .iter()
//                 .flat_map(|(start, end)| vec![start.color, end.color])
//                 .flatten()
//                 .collect();

//             let normal: Vec<[f32; 3]> = lines
//                 .lines
//                 .iter()
//                 .flat_map(|(start, end)| vec![start.normal, end.normal])
//                 .flatten()
//                 .collect();

//             let joint_indices: Vec<[u16; 4]> = lines
//                 .lines
//                 .iter()
//                 .flat_map(|(start, end)| vec![start.joint_indices, end.joint_indices])
//                 .flatten()
//                 .collect();

//             let joint_weights: Vec<[f32; 4]> = lines
//                 .lines
//                 .iter()
//                 .flat_map(|(start, end)| vec![start.joint_weights, end.joint_weights])
//                 .flatten()
//                 .collect();

//             let joint_indices_count = joint_indices.len();
//             let joint_weights_count = joint_weights.len();

//             println!("{} {}", joint_indices_count, joint_weights_count);

//             assert_eq!(
//                 joint_indices_count, joint_weights_count,
//                 "Joint indices and weights must have the same length"
//             );

//             line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
//             line_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
//             line_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normal);
//             // line_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_INDEX,VertexAttributeValues::Uint16x4(joint_indices));
//             line_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weights);

//             let line_mesh_handle = meshes.add(line_mesh);

//             // commands.entity(entity).with_children(|parent| {
//             //     parent.spawn(MaterialMeshBundle {
//             //         mesh: line_mesh_handle,
//             //         // material: line_material,
//             //         material: standard_materials.add(StandardMaterial::default()),
//             //         transform: Transform::from_matrix(global_transform.compute_matrix())
//             //             .with_scale(Vec3::splat(10.)),
//             //         ..default()
//             //     });
//             // });

//             commands.entity(entity).remove::<Handle<StandardMaterial>>();
//         }
//     }
// }

// fn process_scene_old(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut fill_materials: ResMut<Assets<FillMaterial>>,
//     mut outline_materials: ResMut<Assets<OutlineMaterial>>,
//     mut standard_materials: ResMut<Assets<StandardMaterial>>,
//     mut line_materials: ResMut<Assets<LineMaterial>>,
//     query: Query<
//         (
//             Entity,
//             &Handle<StandardMaterial>,
//             &Handle<Mesh>,
//             &GlobalTransform,
//         ),
//         Added<Handle<Mesh>>,
//     >,
// ) {
//     for (entity, _material_handle, mesh_handle, global_transform) in query.iter() {
//         if let Some(mesh) = meshes.get_mut(mesh_handle) {
//             random_color_mesh(mesh);
//             smooth_normals(mesh);

//             // Create new materials
//             let fill_material = fill_materials.add(FillMaterial {
//                 color: Vec4::new(0.0, 0.0, 0.0, 1.0),
//                 displacement: 10.0,
//             });

//             let outline_material = outline_materials.add(OutlineMaterial {
//                 flat_color: Vec4::new(0.0, 1.0, 1.0, 1.0),
//                 outline_width: 0.1,
//                 ..default()
//             });

//             let line_material = line_materials.add(LineMaterial {});
//             let line_list = generate_edge_line_list(mesh);

//             // let positions: Vec<[f32; 3]> = lines
//             //     .lines
//             //     .iter()
//             //     .flat_map(|(start, end)| vec![start.position.to_array(), end.position.to_array()])
//             //     .collect();

//             let colors: Vec<[f32; 4]> = line_list
//                 .lines
//                 .iter()
//                 .flat_map(|(start, end)| vec![start.color.to_array(), end.color.to_array()])
//                 .collect();

//             let mut line_mesh = Mesh::new(
//                 bevy::render::render_resource::PrimitiveTopology::LineList,
//                 RenderAssetUsages::RENDER_WORLD,
//             );

//             let positions: &VertexAttributeValues =
//                 mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();

//             if let (Some(VertexAttributeValues::Float32x3(pp)),) =
//                 (mesh.attribute(Mesh::ATTRIBUTE_POSITION),)
//             {
//                 let mut set = HashSet::new();
//                 for line in line_list.lines {
//                     let print_it = |p1: [f32; 3], p2: [f32; 3]| {
//                         println!(
//                             "{}:{}:{}  -> {}:{}:{}",
//                             p1[0], p1[1], p1[2], p2[0], p2[1], p2[2]
//                         );
//                     };

//                     let edge: IntEdge = if line.0.index < line.1.index {
//                         IntEdge::new_from_floats(
//                             pp[line.0.index as usize],
//                             pp[line.1.index as usize],
//                         )
//                     } else {
//                         IntEdge::new_from_floats(
//                             pp[line.1.index as usize],
//                             pp[line.0.index as usize],
//                         )
//                     };

//                     set.insert(edge);

//                     print_it(pp[line.0.index as usize], pp[line.1.index as usize])
//                 }

//                 println!("Unique lines: {}", set.len());
//             }

//             line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
//             // line_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
//             let line_mesh_handle = meshes.add(line_mesh);

//             commands.entity(entity).with_children(|parent| {
//                 parent.spawn(MaterialMeshBundle {
//                     mesh: line_mesh_handle,
//                     material: line_material,
//                     transform: Transform::from_matrix(global_transform.compute_matrix()), // .with_scale(Vec3::splat(10.))
//                     ..default()
//                 });
//             });

//             // Remove the StandardMaterial component
//             commands.entity(entity).remove::<Handle<StandardMaterial>>();

//             // // Add the new material components to the original mesh
//             // commands
//             //     .entity(entity)
//             //     .insert(fill_material)
//             //     .insert(outline_material);
//         }
//     }
// }

// fn count_unique_lines(line_list: &LineList, ps: &VertexAttributeValues) -> usize {

//     let p = VertexAttributeValues::Float32x3(ps);

//     let mut set = HashSet::new();
//     for line in line_list.lines.iter() {
//         let edge = if line.0.index < line.1.index {
//             (p[line.0.index as usize], p[line.1.index as usize])
//         } else {
//             (p[line.1.index as usize], p[line.0.index as usize])
//         };
//         set.insert(edge);
//     }
//     set.len()
// }
