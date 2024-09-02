use bevy::ecs::observer::TriggerTargets;
use bevy::gltf::{GltfExtras, GltfSceneExtras};
use bevy::{
    animation::animate_targets,
    asset::AssetMetaCheck,
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    gltf::GltfPlugin,
    prelude::*,
    render::{
        mesh::{skinning::SkinnedMesh, MeshVertexAttribute},
        render_resource::VertexFormat,
    },
    scene::SceneInstanceReady,
};

use serde_json::Value;
use std::collections::HashMap;

use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use fill_material::FillMaterial;
use line_material::LineMaterial;
use mesh_ops::{
    get_smoothed_normals, line_list_to_mesh, mesh_to_line_list_custom, mesh_to_wireframe,
    RandomizeVertexColors,
};
use outline_material::OutlineMaterial;
use std::time::Duration;

mod camera_plugin;
mod fill_material;
mod line_material;
// mod load_json;
mod mesh_ops;
mod outline_material;

// const PATH: &str = "astro/scene.gltf";
const ASTROPATH: &str = "astro_custom/scene.gltf";
// const FOXPATH: &str = "fox.glb";
// const PATH: &str = "sphere_flat.gltf";
// const PATH: &str = "sphere.gltf";
// const PATH: &str = "torus.gltf";
const TORUSPATH: &str = "temp/torus_custom_property.gltf";
const COUPEPATH: &str = "temp/coupe.gltf";

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
        // .add_systems(Update, process_scene)
        .add_systems(Update, extras)
        .add_systems(Update, ui_system) // Add this line
        // .add_systems(Update, ui_example_system)  // Add this line
        // .add_systems(Update, check_extras)
        // .add_systems(Update, check_for_gltf_extras)
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
    let mut graph = AnimationGraph::new();
    let animations = graph
        .add_clips(
            [GltfAssetLabel::Animation(0).from_asset(COUPEPATH)]
                // [GltfAssetLabel::Animation(0).from_asset(ASTROPATH)]
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

    // let astro = commands
    //     .spawn((
    //         SceneBundle {
    //             scene: assets.load(GltfAssetLabel::Scene(0).from_asset(ASTROPATH)),
    //             transform: Transform::from_xyz(0.0, -1.2, 0.0)
    //                 .with_rotation(Quat::from_rotation_y(0.0))
    //                 .with_scale(Vec3::splat(1.)),
    //             ..default()
    //         },
    //         WireframeSettings {
    //             // gltf_path: None,
    //             // gltf_path: Some(String::from(ASTROPATH)),

    //         },
    //     ))
    //     .id();

    // let torus = commands
    //     .spawn((
    //         SceneBundle {
    //             scene: assets.load(GltfAssetLabel::Scene(0).from_asset(TORUSPATH)),
    //             transform: Transform::from_xyz(0.0, 0.0, 0.0)
    //                 .with_rotation(Quat::from_rotation_y(0.0))
    //                 .with_scale(Vec3::splat(1.)),
    //             ..default()
    //         },
    //         WireframeSettings {},
    //     ))
    //     .id();

    let coupe = commands
        .spawn((
            SceneBundle {
                scene: assets.load(GltfAssetLabel::Scene(0).from_asset(COUPEPATH)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_y(0.0))
                    .with_scale(Vec3::splat(1.)),
                ..default()
            },
            WireframeSettings {},
        ))
        .id();
}

fn extras(
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
) {
    for event in events.read() {
        let mut parsed_edges: Option<HashMap<String, JsonLineList>> = None;

        for e in children.iter_descendants(event.parent) {
            if let Ok(scene_extras) = scene_extras.get(e) {
                if let Some(parsed) = parse_gltf_extra(&scene_extras.value) {
                    parsed_edges = Some(parsed);
                    break; // Assuming you only need to parse this once
                }
            }
        }

        println!("a");
        
        for entity in children.iter_descendants(event.parent) {
            if let Ok((mesh_handle, parent)) = mesh.get(entity) {
                if let Some(flat_mesh) = mesh_assets.get_mut(mesh_handle) {
                    
                    println!("b");
                    
                    commands.entity(entity).remove::<Handle<StandardMaterial>>();

                    flat_mesh.randomize_vertex_colors();
                    let smoothed_normals = get_smoothed_normals(flat_mesh).unwrap();
                    flat_mesh.insert_attribute(ATTRIBUTE_SMOOTHED_NORMAL, smoothed_normals);

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

                    // let custom_line_list = None;
                    if let Ok(w) = wf.get(event.parent) {
                        println!("c");
                        if let Ok(extras) = extras.get(parent.get()) {
                            println!("d");
                            if let Ok(json_value) = serde_json::from_str::<Value>(&extras.1.value) {
                                println!("e");
                                if let Some(gltf_primitive_index) =
                                    json_value.get("gltf_primitive_index")
                                {
                                    println!("f");
                                    let key = gltf_primitive_index.to_string();

                                    // Safely access parsed_edges
                                    match &parsed_edges {
                                        Some(extras) => {
                                            if let Some(parsed_edge) = extras.get(&key) {
                                                println!("g");
                                                let smooth_mesh = flat_mesh.clone();

                                                let line_list =
                                                    flat_mesh.mesh_to_line_list_custom(parsed_edge);
                                                let line_mesh =
                                                    line_list_to_mesh(&line_list, &smooth_mesh);

                                                let new_mesh_handle = mesh_assets.add(line_mesh);
                                                let skinned_mesh =
                                                    skinned_meshes.get(entity).cloned();

                                                let bundle = MaterialMeshBundle {
                                                    mesh: new_mesh_handle,
                                                    material: line_materials.add(LineMaterial {
                                                        displacement: 1.5,
                                                        ..default()
                                                    }),
                                                    ..Default::default()
                                                };
                                    
                                                
                                                // let mut entity_commands = commands.spawn(bundle);
                                    
                                                // // If the original entity had a SkinnedMesh component, add it to the new entity
                                                // if let Ok(skinned_mesh) = skinned_mesh {
                                                //     entity_commands.insert(skinned_mesh);
                                                // }
                                    
                                    
                                                commands.entity(entity).with_children(|parent| {
                                                    let mut child_entity = parent.spawn(bundle);
                                                    
                                                    // If the original entity had a SkinnedMesh component, add it to the new entity
                                                    if let Ok(skinned_mesh) = skinned_mesh {
                                                        child_entity.insert(skinned_mesh);
                                                    }
                                                });
                                                
                                            }
                                        }
                                        None => {}
                                    }
                                }
                            } else {
                                warn!("Failed to parse JSON in extras");
                            }
                        }
                    }
                }
            }
        }
    }
}

fn process_scene(
    mut commands: Commands,
    mut events: EventReader<SceneInstanceReady>,
    children: Query<&Children>,
    // parent: &Parent,
    meshes: Query<(Entity, &Handle<Mesh>)>,
    skinned_meshes: Query<&SkinnedMesh>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    mut fill_materials: ResMut<Assets<FillMaterial>>, // Add FillMaterial resource
    mut outline_materials: ResMut<Assets<OutlineMaterial>>, // Add FillMaterial resource
    shader_settings: Res<ShaderSettings>,
    processable_scenes: Query<&WireframeSettings>,
    gltf_extras: Query<(Entity, &GltfExtras)>, // Modified this line
    gltf_scene_extras: Query<(Entity, &GltfSceneExtras)>,
) {
    // for event in events.read() {
    //     if !processable_scenes.contains(event.parent) {
    //         continue;
    //     }
    //     println!("a");

    //     if let Ok(r) = gltf_extras.get(event.parent) {
    //         // println!("b");

    //         // println!("GLTF EXTRAS {:?}", r);
    //     }

    //     // let parsed_extra = None;

    //     // for (e, x) in gltf_extras.iter() {
    //     //     let z = gltf_extras.get(e);

    //     //     println!("Z {:?}", z);
    //     // }

    //     // for e in children.iter_descendants(event.parent) {
    //     //     // println!("Entity: {:?}", e);
    //     //     let z = gltf_extras.get(e);
    //     //     if z.is_ok() {
    //     //         // println!("Z {:?}", z);
    //     //         println!("Entity: {:?}", e);
    //     //         for se in children.iter_descendants(e){
    //     //             println!("  Entity: {:?}", se);
    //     //         }
    //     //     }
    //     // }

    //     let mut parsed_edges = None;

    //     for entity in children.iter_descendants(event.parent) {
    //         if let Ok((_, extras)) = gltf_scene_extras.get(entity) {
    //             parsed_edges = Some(parse_selected_edges(&extras.value).unwrap_or_default());
    //             //todo handle error
    //         }
    //     }

    //     println!("b");

    //     for entity in children.iter_descendants(event.parent) {
    //         if let (Ok((entity, mesh_handle)), Ok(wireframe_settings)) =
    //             (meshes.get(entity), processable_scenes.get(event.parent))
    //         {
    //             if let Some(flat_mesh) = mesh_assets.get_mut(mesh_handle) {
    //                 println!("Entity: {:?}", entity);

    //                 commands.entity(entity).remove::<Handle<StandardMaterial>>();
    //                 flat_mesh.randomize_vertex_colors();

    //                 let smoothed_normals = get_smoothed_normals(flat_mesh).unwrap();
    //                 flat_mesh.insert_attribute(ATTRIBUTE_SMOOTHED_NORMAL, smoothed_normals);

    //                 let mut smooth_mesh = flat_mesh.clone();

    //                 // smooth_mesh.compute_smooth_normals();
    //                 // smooth_mesh.smooth_normals_non_indexed();
    //                 flat_mesh.duplicate_vertices();
    //                 flat_mesh.compute_flat_normals();
    //                 // flat_mesh.compute_normals();

    //                 // Add FillMaterial component
    //                 let fill_material_handle = fill_materials.add(FillMaterial {
    //                     color: Vec4::new(0.0, 0.0, 0.0, 1.0),
    //                     displacement: 0.0,
    //                     shininess: 200.0,
    //                     specular_strength: 1.0,
    //                 });
    //                 commands.entity(entity).insert(fill_material_handle.clone());

    //                 // Add OutlineMaterial component
    //                 let outline_material_handle = outline_materials.add(OutlineMaterial {
    //                     outline_width: shader_settings.outline_width,
    //                     ..default()
    //                 });
    //                 commands
    //                     .entity(entity)
    //                     .insert(outline_material_handle.clone());

    //                 // let custom_line_list = None;
    //                 match mesh_to_wireframe(&mut smooth_mesh, &wireframe_settings, &parsed_edges) {
    //                     Ok(_) => {}
    //                     Err(e) => {
    //                         panic!("fuckkkkkkkk");
    //                         // warn!("Error: {:?}", e);
    //                     }
    //                 }
    //                 // mesh_to_wireframe(&mut smooth_mesh, &wireframe_settings);

    //                 let new_mesh_handle = mesh_assets.add(smooth_mesh);
    //                 let skinned_mesh = skinned_meshes.get(entity).cloned();

    //                 let bundle = MaterialMeshBundle {
    //                     mesh: new_mesh_handle,
    //                     material: line_materials.add(LineMaterial {
    //                         displacement: 1.5,
    //                         ..default()
    //                     }),
    //                     ..Default::default()
    //                 };
    //             }
    //         }
    //     }
    // }
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
            egui::Slider::new(&mut shader_settings.wireframe_displacement, 0.0..=1.0)
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

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct GltfExtra {
    selected_edges_json: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct JsonLineList {
    pub line_list: Vec<[u32; 2]>,
}

impl From<Vec<Vec<i32>>> for JsonLineList {
    fn from(edges: Vec<Vec<i32>>) -> Self {
        let line_list = edges
            .into_iter()
            .map(|e| [e[0] as u32, e[1] as u32])
            .collect();
        JsonLineList { line_list }
    }
}

fn parse_gltf_extra(json_str: &str) -> Option<HashMap<String, JsonLineList>> {
    serde_json::from_str::<Value>(json_str)
        .ok()
        .and_then(|json_value| {
            json_value
                .get("gltf_all_selected_edges")
                .and_then(|edges_str| {
                    serde_json::from_str::<Value>(edges_str.as_str()?)
                        .ok()
                        .map(|edges_json| {
                            let mut result = HashMap::new();
                            if let Some(edges_obj) = edges_json.as_object() {
                                for (key, value) in edges_obj {
                                    if let Some(edge_array) = value.as_array() {
                                        let line_list: Vec<[u32; 2]> = edge_array
                                            .iter()
                                            .filter_map(|pair| {
                                                if let Some(pair_array) = pair.as_array() {
                                                    if pair_array.len() == 2 {
                                                        Some([
                                                            pair_array[0].as_u64()? as u32,
                                                            pair_array[1].as_u64()? as u32,
                                                        ])
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect();
                                        result.insert(key.clone(), JsonLineList { line_list });
                                    }
                                }
                            }
                            result
                        })
                })
        })
}

// fn parse_gltf_extra(json_str: &str) -> Result<JsonLineList, serde_json::Error> {
//     let gltf_extra: GltfExtra = serde_json::from_str(json_str)?;

//     let edges: Vec<Vec<i32>> = serde_json::from_str(&gltf_extra.selected_edges_json)?;

//     Ok(JsonLineList::from(edges))
// }

fn parse_selected_edges(
    json_str: &str,
) -> Result<HashMap<String, JsonLineList>, serde_json::Error> {
    let parsed: Value = serde_json::from_str(json_str)?;
    let mut result = HashMap::new();

    if let Value::Object(obj) = parsed {
        if let Some(Value::String(edges_str)) = obj.get("gltf_all_selected_edges") {
            let edges: HashMap<String, Vec<[u32; 2]>> = serde_json::from_str(edges_str)?;
            for (key, value) in edges {
                result.insert(key, JsonLineList { line_list: value });
            }
        }
    }

    Ok(result)
}
