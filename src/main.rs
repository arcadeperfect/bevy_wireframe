use bevy::{
    animation::animate_targets,
    gltf::GltfPlugin,
    prelude::*,
    render::{
        mesh::{skinning::SkinnedMesh, MeshVertexAttribute},
        render_resource::VertexFormat,
    },
    scene::SceneInstanceReady,
};
use fill_material::FillMaterial;
use line_material::LineMaterial;
use load_json::jparse;
use mesh_ops::{
    generate_edge_line_list, line_list_to_mesh, random_color_mesh, smooth_normals
};
use outline_material::OutlineMaterial;
use std::time::Duration;

mod camera_plugin;
mod fill_material;
mod line_material;
mod mesh_ops;
mod outline_material;
mod load_json;

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

// const ATTRIBUTE_CUSTOM: MeshVertexAttribute =
//     MeshVertexAttribute::new("Custom", 2137464976, VertexFormat::Uint32);

const ATTRIBUTE_CUSTOM: MeshVertexAttribute =
    MeshVertexAttribute::new("Custom", 2137464976, VertexFormat::Float32);

const ATTRIBUTE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("Index", 1237464976, VertexFormat::Float32);

fn main() {

    App::new()
        // .insert_resource(AmbientLight {
        //     color: Color::WHITE,
        //     brightness: 2000.,
        // })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins.set(
                GltfPlugin::default()
                    // Map a custom glTF attribute name to a `MeshVertexAttribute`.
                    .add_custom_vertex_attribute("CUSTOM", ATTRIBUTE_CUSTOM)
                    .add_custom_vertex_attribute("INDEX", ATTRIBUTE_INDEX),
            ),
        )
        .add_plugins(camera_plugin::CamPlugin)
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

    let disco_naut_1 = SceneBundle {
        scene: assets.load(GltfAssetLabel::Scene(0).from_asset(PATH)),
        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(0.0)),
        ..default()
    };
    let entity_1 = commands.spawn((disco_naut_1, OriginalSceneTag)).id();
    
    // let disco_naut_2 = SceneBundle {
    //     scene: assets.load(GltfAssetLabel::Scene(0).from_asset(PATH)),
    //     transform: Transform::from_xyz(1.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(0.0)),
    //     ..default()
    // };
    // let entity_2 = commands.spawn((disco_naut_2, OriginalSceneTag)).id();

    commands.insert_resource(MyScene(entity_1));
}

fn system(
    mut commands: Commands,
    mut events: EventReader<SceneInstanceReady>,
    // my_scene_entity: Res<MyScene>,
    children: Query<&Children>,
    meshes: Query<(Entity, &Handle<Mesh>)>,
    skinned_meshes: Query<&SkinnedMesh>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    mut fill_materials: ResMut<Assets<FillMaterial>>, // Add FillMaterial resource
    mut outline_materials: ResMut<Assets<OutlineMaterial>>, // Add FillMaterial resource
) {
    for event in events.read() {
        // if event.parent == my_scene_entity.0 {
        if true {
            for entity in children.iter_descendants(event.parent) {
                if let Ok((entity, mesh_handle)) = meshes.get(entity) {
                    if let Some(original_mesh) = mesh_assets.get_mut(mesh_handle) {
                        commands.entity(entity).remove::<Handle<StandardMaterial>>();

                        random_color_mesh(original_mesh);
                        smooth_normals(original_mesh);

                        // Add FillMaterial component
                        let fill_material_handle = fill_materials.add(FillMaterial {
                            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                            displacement: 0.1,
                        });
                        commands.entity(entity).insert(fill_material_handle.clone());

                        // Add OutlineMaterial component
                        let outline_material_handle = outline_materials.add(OutlineMaterial {
                            outline_width: 0.05,
                            ..default()
                        });
                        commands
                            .entity(entity)
                            .insert(outline_material_handle.clone());

                        let mut new_mesh = original_mesh.clone();
                        smooth_normals(&mut new_mesh);
                        mesh_to_wireframe(&mut new_mesh);
                        let new_mesh_handle = mesh_assets.add(new_mesh);
                        let skinned_mesh = skinned_meshes.get(entity).cloned();

                        let mut bundle = MaterialMeshBundle {
                            mesh: new_mesh_handle,
                            material: line_materials.add(LineMaterial{
                                displacement: 1.5,
                                ..default()
                            }),
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
    random_color_mesh(mesh);
    // smooth_normals(mesh);

    let line_list = generate_edge_line_list(&mesh, true);
    let line_mesh = line_list_to_mesh(&line_list, mesh);

    *mesh = line_mesh;
}

fn play_animation_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
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

