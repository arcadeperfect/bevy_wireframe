use bevy::gltf::{GltfExtras, GltfSceneExtras};
use bevy::{
    animation::animate_targets,
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    gltf::GltfPlugin,
    prelude::*,
    render::{
        mesh::{skinning::SkinnedMesh, MeshVertexAttribute},
        render_resource::VertexFormat,
    },
    scene::SceneInstanceReady,
};

use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use fill_material::FillMaterial;
use line_material::LineMaterial;
use mesh_ops::{get_smoothed_normals, line_list_to_mesh, MeshToLineList, VertexOps};
use outline_material::OutlineMaterial;
use parse_extras::{parse_gltf_extra_json, JsonLineList};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

mod camera_plugin;
mod fill_material;
mod line_material;
mod mesh_ops;
mod outline_material;
mod parse_extras;

// const ASTROPATH: &str = "astro/scene.gltf";
const ASTRO_PATH: &str = "temp/astro.gltf";
// const ASTROPATH: &str = "astro_custom/scene.gltf";
// const FOXPATH: &str = "fox.glb";
// const PATH: &str = "sphere_flat.gltf";
// const PATH: &str = "sphere.gltf";
// const PATH: &str = "torus.gltf";
const TORUS_PATH: &str = "temp/torus.gltf";
const COUPE_PATH: &str = "temp/coupe2.gltf";
const SPHERE_NO_EXTRAS_PATH: &str = "temp/sphere_no_extras.gltf";
// const COUPE_PATH: &str = "temp/discovery.gltf";

// #[derive(Resource)]
// struct MyScene(Entity);

// #[derive(Component)]
// struct WireFrameScene;

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    graph: Handle<AnimationGraph>,
}

#[derive(Resource)]
struct ShaderSettings {
    outline_width: f32,
    wireframe_displacement: f32,
    fill_displacement: f32,
    fill_shininess: f32,
    fill_specular_strength: f32,
}

impl Default for ShaderSettings {
    fn default() -> Self {
        Self {
            outline_width: 0.1,
            wireframe_displacement: 0.0,
            fill_displacement: 0.0,
            fill_shininess: 250.0,
            fill_specular_strength: 0.1,
        }
    }
}

#[derive(Component)]
struct WireframeSettings {
    // gltf_path: Option<String>,
}

const ATTRIBUTE_VERT_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("VERT_INDEX", 1237464976, VertexFormat::Float32);

const ATTRIBUTE_SMOOTHED_NORMAL: MeshVertexAttribute =
    MeshVertexAttribute::new("SmoothNormal", 723495149, VertexFormat::Float32x3);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(ShaderSettings::default())
        .add_plugins(
            DefaultPlugins.set(
                GltfPlugin::default()
                    .add_custom_vertex_attribute("VERT_INDEX", ATTRIBUTE_VERT_INDEX)
                    .add_custom_vertex_attribute("SMOOTH_NORMAL", ATTRIBUTE_SMOOTHED_NORMAL),
            ),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(MaterialPlugin::<FillMaterial>::default())
        .add_plugins(MaterialPlugin::<OutlineMaterial>::default())
        .add_plugins(MaterialPlugin::<LineMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, play_animation_once_loaded.before(animate_targets))
        .add_systems(Update, post_process)
        .add_systems(Update, ui_system) // Add this line
        .run();
}




fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        PanOrbitCamera::default(),
        BloomSettings::NATURAL,
    ));

    // Build the animation graph
    let mut graph1a = AnimationGraph::new();
    let animations1 = graph1a
        .add_clips(
            // [GltfAssetLabel::Animation(0).from_asset(COUPEPATH)]
            [GltfAssetLabel::Animation(0).from_asset(ASTRO_PATH)]
                .into_iter()
                .map(|path| assets.load(path)),
            1.0,
            graph1a.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let graph1b = graphs.add(graph1a);
    commands.insert_resource(Animations {
        animations: animations1,
        graph: graph1b.clone(),
    });

    // // Build the animation graph
    // let mut graph2a = AnimationGraph::new();
    // let animations2 = graph2a
    //     .add_clips(
    //         // [GltfAssetLabel::Animation(0).from_asset(COUPEPATH)]
    //         [GltfAssetLabel::Animation(0).from_asset(COUPE_PATH)]
    //             .into_iter()
    //             .map(|path| assets.load(path)),
    //         1.0,
    //         graph2a.root,
    //     )
    //     .collect();

    // // Insert a resource with the current scene information
    // let graph2b = graphs.add(graph2a);
    // commands.insert_resource(Animations {
    //     animations: animations2,
    //     graph: graph2b.clone(),
    // });
    
    // let coupe = commands
    //     .spawn((
    //         SceneBundle {
    //             scene: assets.load(GltfAssetLabel::Scene(0).from_asset(COUPE_PATH)),
    //             transform: Transform::from_xyz(0.0, 0.0, 0.0)
    //                 .with_rotation(Quat::from_rotation_y(0.0))
    //                 .with_scale(Vec3::splat(1.0)),
    //             ..default()
    //         },
    //         WireframeSettings {},
    //     ))
    //     .id();
    
    
    let astro = commands
        .spawn((
            SceneBundle {
                scene: assets.load(GltfAssetLabel::Scene(0).from_asset(ASTRO_PATH)),
                transform: Transform::from_xyz(0.0, -1.2, 0.0)
                    .with_rotation(Quat::from_rotation_y(0.0))
                    .with_scale(Vec3::splat(1.)),
                ..default()
            },
            WireframeSettings {},
        ))
        .id();

    // let torus = commands
    //     .spawn((
    //         SceneBundle {
    //             scene: assets.load(GltfAssetLabel::Scene(0).from_asset(TORUS_PATH)),
    //             transform: Transform::from_xyz(0.0, 0.0, 0.0)
    //                 .with_rotation(Quat::from_rotation_y(0.0))
    //                 .with_scale(Vec3::splat(1.)),
    //             ..default()
    //         },
    //         WireframeSettings {},
    //     ))
    //     .id();

    // let torus2 = commands
    //     .spawn((SceneBundle {
    //         scene: assets.load(GltfAssetLabel::Scene(0).from_asset(TORUS_PATH)),
    //         transform: Transform::from_xyz(1.0, 0.0, 0.0)
    //             .with_rotation(Quat::from_rotation_y(0.0))
    //             .with_scale(Vec3::splat(1.)),
    //         ..default()
    //     },))
    //     .id();

    // let sphere_no_extras = commands
    //     .spawn((
    //         SceneBundle {
    //             scene: assets.load(GltfAssetLabel::Scene(0).from_asset(SPHERE_NO_EXTRAS_PATH)),
    //             transform: Transform::from_xyz(1.0, 0.0, 0.0)
    //                 .with_rotation(Quat::from_rotation_y(0.0))
    //                 .with_scale(Vec3::splat(1.)),
    //             ..default()
    //         },
    //         WireframeSettings {},
    //     ))
    //     .id();

   
}

#[derive(Component)]
struct WheelRotator {
    rotation_speed: f32,
}

fn rotate_wheels(time: Res<Time>, mut query: Query<(&WheelRotator, &mut Transform)>) {
    for (wheel, mut transform) in query.iter_mut() {
        transform.rotate_x(wheel.rotation_speed * time.delta_seconds());
    }
}

fn post_process(
    mut commands: Commands,
    mut events: EventReader<SceneInstanceReady>,
    extras: Query<(&Parent, &GltfExtras)>,
    scene_extras: Query<&GltfSceneExtras>,
    mesh: Query<(&Handle<Mesh>, &Parent)>,
    children: Query<&Children>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    mut fill_materials: ResMut<Assets<FillMaterial>>,
    mut outline_materials: ResMut<Assets<OutlineMaterial>>, // Add FillMaterial resource
    mut mesh_assets: ResMut<Assets<Mesh>>,
    shader_settings: Res<ShaderSettings>,
    wf: Query<&WireframeSettings>,
    skinned_meshes: Query<&SkinnedMesh>,
    query: Query<(Entity, &Name), Without<WheelRotator>>,
) {
    for (entity, name) in query.iter() {
        if name.to_lowercase().contains("wheel") {
            commands.entity(entity).insert(WheelRotator {
                rotation_speed: 50.0, // Adjust this value to change rotation speed
            });
        }
    }

    for event in events.read() {
        // Only proceeed if the spawned mesh has a wireframe component

        if wf.get(event.parent).is_err() {
            continue;
        }

        // Out of laziness I iterate through the whole scene until I find the scene level extra which contains a json dictionary
        // that encodes the line lists generated in blender, with the index of the mesh as the key
        // there will only be one of these, so once it finds it, it parses it and breaks the loop
        // todo: better way to locate the scene level extra

        let mut line_list_data_from_blender: Option<HashMap<String, JsonLineList>> = None;

        for e in children.iter_descendants(event.parent) {
            if let Ok(loaded_scene_extras) = scene_extras.get(e) {
                if let Some(parsed) = parse_gltf_extra_json(&loaded_scene_extras.value) {
                    line_list_data_from_blender = Some(parsed);
                    break; // Assuming you only need to parse this once
                }
            }
        }

        // Now we iterate through each mesh and apply the wireframe post processing
        // If scene extras were found, it will generate the line lists according to the json dictionary
        // Otherwise it will generate a line list for every edge of the mesh

        for entity in children.iter_descendants(event.parent) {
            if let Ok((mesh_handle, parent)) = mesh.get(entity) {
                if let Some(mesh) = mesh_assets.get_mut(mesh_handle) {
                    commands.entity(entity).remove::<Handle<StandardMaterial>>();

                    mesh.randomize_vertex_colors(); //todo problem is this effects other instances of the mesh in scenes that don't have the wireframe component

                    let smoothed_normals: Vec<[f32; 3]> = get_smoothed_normals(mesh).unwrap();
                    // invert_normals(&mut smoothed_normals);
                    mesh.insert_attribute(ATTRIBUTE_SMOOTHED_NORMAL, smoothed_normals);

                    mesh.duplicate_vertices();
                    mesh.compute_flat_normals();

                    // Add FillMaterial component
                    let fill_material_handle = fill_materials.add(FillMaterial {
                        color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                        displacement: 0.0,
                        shininess: 200.0,
                        specular_strength: 1.0,
                    });
                    commands.entity(entity).insert(fill_material_handle.clone());

                    // Add OutlineMaterial component
                    let outline_material_handle = outline_materials.add(OutlineMaterial {
                        outline_width: shader_settings.outline_width,
                        ..default()
                    });
                    commands
                        .entity(entity)
                        .insert(outline_material_handle.clone());

                    // To use the json line lists we need an index for each mesh which is encoded as a mesh level extra
                    // If this is succesfully parsed, and if the json dictionary was parsed and contains the key, we can use the line list
                    // Otherwise we generate a line list for every edge of the mesh

                    let mut parsed_line_list: Option<&JsonLineList> = None;

                    match &line_list_data_from_blender {
                        Some(blender_data) => {
                            if let Ok(mesh_extra) = extras.get(parent.get()) {
                                if let Ok(json_value) =
                                    serde_json::from_str::<Value>(&mesh_extra.1.value)
                                {
                                    if let Some(key) = json_value.get("gltf_primitive_index") {
                                        if let Some(parsed_edge) =
                                            blender_data.get(&key.to_string())
                                        {
                                            parsed_line_list = Some(parsed_edge);
                                        } else {
                                            warn!("key not found in json line list data");
                                        }
                                    } else {
                                        warn!("no key found for this primitive");
                                    }
                                } else {
                                    warn!("Unable to parse json");
                                }
                            }
                        }
                        None => {
                            info!("No line list data found for mesh, all triangles will be used");
                        }
                    }

                    // LineList stores the data required to build a mesh of lines
                    // It can be derived from gltf extra data, or generated for every triangle in the absence

                    let line_list;

                    match parsed_line_list {
                        Some(p) => {
                            line_list = mesh.mesh_to_line_list_custom_data(p);
                        }
                        None => {
                            line_list = mesh.mesh_to_line_list();
                        }
                    }

                    let line_mesh = line_list_to_mesh(&line_list, &mesh);
                    let new_mesh_handle = mesh_assets.add(line_mesh);
                    let skinned_mesh = skinned_meshes.get(entity).cloned(); // required for scenes with skinned mesh animations

                    let bundle = MaterialMeshBundle {
                        mesh: new_mesh_handle,
                        material: line_materials.add(LineMaterial {
                            displacement: 1.5,
                            ..default()
                        }),
                        ..Default::default()
                    };

                    commands.entity(entity).with_children(|parent| {
                        let mut child_entity = parent.spawn(bundle);

                        // If the original entity had a SkinnedMesh component, add it to the new entity
                        if let Ok(skinned_mesh) = skinned_mesh {
                            child_entity.insert(skinned_mesh);
                        }
                    });
                }
            }
        }
    }
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

fn ui_system(
    mut contexts: EguiContexts,
    mut shader_settings: ResMut<ShaderSettings>,
    mut outline_materials_assets: ResMut<Assets<OutlineMaterial>>,
    outline_materials: Query<&Handle<OutlineMaterial>>,
    mut line_materials_assets: ResMut<Assets<LineMaterial>>,
    line_materials: Query<&Handle<LineMaterial>>,
    mut fill_materials_assets: ResMut<Assets<FillMaterial>>,
    fill_materials: Query<&Handle<FillMaterial>>,
) {
    egui::Window::new("Shader Controls").show(contexts.ctx_mut(), |ui| {
        ui.add(
            egui::Slider::new(&mut shader_settings.outline_width, 0.0..=1.0).text("Outline Width"),
        );
        ui.add(
            egui::Slider::new(&mut shader_settings.wireframe_displacement, 0.0..=5.0)
                .text("Wireframe Displacement"),
        );
        ui.add(
            egui::Slider::new(&mut shader_settings.fill_displacement, 0.0..=1.0)
                .text("Fill Displacement"),
        );
        ui.add(
            egui::Slider::new(&mut shader_settings.fill_shininess, 1.0..=256.0).text("Shininess"),
        );
        ui.add(
            egui::Slider::new(&mut shader_settings.fill_specular_strength, 0.0..=1.0)
                .text("Specular Strength"),
        );
    });

    // Update all OutlineMaterial instances
    for material_handle in outline_materials.iter() {
        if let Some(material) = outline_materials_assets.get_mut(material_handle) {
            material.outline_width = shader_settings.outline_width;
        }
    }

    // Update all LineMaterial instances
    for material_handle in line_materials.iter() {
        if let Some(material) = line_materials_assets.get_mut(material_handle) {
            material.displacement = shader_settings.wireframe_displacement;
        }
    }

    // Update all FillMaterial instances
    for material_handle in fill_materials.iter() {
        if let Some(material) = fill_materials_assets.get_mut(material_handle) {
            material.displacement = shader_settings.fill_displacement;
            material.shininess = shader_settings.fill_shininess;
            material.specular_strength = shader_settings.fill_specular_strength;
        }
    }
}

// #[derive(serde::Deserialize, serde::Serialize, Debug)]
// struct GltfExtra {
//     selected_edges_json: String,
// }

impl From<Vec<Vec<i32>>> for JsonLineList {
    fn from(edges: Vec<Vec<i32>>) -> Self {
        let line_list = edges
            .into_iter()
            .map(|e| [e[0] as u32, e[1] as u32])
            .collect();
        JsonLineList { line_list }
    }
}

// fn parse_gltf_extra(json_str: &str) -> Result<JsonLineList, serde_json::Error> {
//     let gltf_extra: GltfExtra = serde_json::from_str(json_str)?;

//     let edges: Vec<Vec<i32>> = serde_json::from_str(&gltf_extra.selected_edges_json)?;

//     Ok(JsonLineList::from(edges))
// }
