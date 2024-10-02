use bevy::gltf::{GltfExtras, GltfSceneExtras};
use bevy::prelude::Color;
use bevy::render::mesh::VertexAttributeValues;
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
use mesh_ops::{
    generate_random_vertex_colors, get_smoothed_normals, line_list_to_mesh, AsFloat4,
    MeshToLineList, VertexOps,
};
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

const ASTRO_PATH: &str = "gltf/astro.gltf";
const TORUS_PATH: &str = "gltf/torus.gltf";
const COUPE_PATH: &str = "gltf/coupe.gltf";
const SPHERE_PATH: &str = "gltf/sphere.gltf";

#[derive(Resource, PartialEq)]
enum VisibleModel {
    Astro,
    Coupe,
    Torus,
    Sphere,
}

// #[derive(Component)]
// struct ColorBuffer{
//     colorz: Option<Vec<Vec<[f32; 4]>>>
// }

// impl Default for ColorBuffer{
//     fn default() -> Self {
//         Self { colorz: None }
//     }
// }

#[derive(Component)]
struct AstroSceneTag;

#[derive(Component)]
struct CoupeSceneTag;
#[derive(Component)]
struct SphereSceneTag;
#[derive(Component)]
struct TorusSceneTag;
#[derive(Resource)]
struct Animations {
    astro_animations: Vec<AnimationNodeIndex>,
    astro_graph: Handle<AnimationGraph>,
    coupe_animations: Vec<AnimationNodeIndex>,
    coupe_graph: Handle<AnimationGraph>,
}

#[derive(Resource)]
struct ShaderSettings {
    outline_width: f32,
    wireframe_displacement: f32,
    fill_displacement: f32,
    fill_shininess: f32,
    fill_specular_strength: f32,
    brightness: f32,
    vertex_color_mode: i32,
    color: Color,
}

impl Default for ShaderSettings {
    fn default() -> Self {
        Self {
            outline_width: 0.1,
            wireframe_displacement: 0.0,
            fill_displacement: 0.0,
            fill_shininess: 250.0,
            fill_specular_strength: 0.1,
            brightness: 15.0,
            vertex_color_mode: 1,
            color: Color::WHITE,
        }
    }
}

#[derive(Component)]
struct WireframeSettings {
    original_colors: Vec<[f32; 4]>,
    // alt_colors: Vec<[f32; 4]>, // TODO add an alt colors system
    current_mode: i32,
}

impl Default for WireframeSettings {
    fn default() -> Self {
        Self {
            original_colors: Vec::new(),
            // alt_colors: Vec::new(),
            current_mode: 1, // Default to vertex color mode
        }
    }
}

const ATTRIBUTE_VERT_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("VERT_INDEX", 1237464976, VertexFormat::Float32);

const ATTRIBUTE_SMOOTHED_NORMAL: MeshVertexAttribute =
    MeshVertexAttribute::new("SmoothNormal", 723495149, VertexFormat::Float32x3);

// const ATTRIBUTE_ALT_COLOR: MeshVertexAttribute =
//     MeshVertexAttribute::new("AltColor", 1948574392, VertexFormat::Float32x4);

fn main() {
    App::new()
        .insert_resource(VisibleModel::Astro)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(ShaderSettings::default())
        .add_plugins(
            DefaultPlugins.set(
                GltfPlugin::default()
                    .add_custom_vertex_attribute("VERT_INDEX", ATTRIBUTE_VERT_INDEX)
                    .add_custom_vertex_attribute("SMOOTH_NORMAL", ATTRIBUTE_SMOOTHED_NORMAL), // .add_custom_vertex_attribute("ALT_COLOR", ATTRIBUTE_ALT_COLOR),
            ),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(MaterialPlugin::<FillMaterial>::default())
        .add_plugins(MaterialPlugin::<OutlineMaterial>::default())
        .add_plugins(MaterialPlugin::<LineMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn)
        .add_systems(Update, play_animation_once_loaded.before(animate_targets))
        .add_systems(Update, post_process)
        .add_systems(Update, ui_system) // Add this line
        // .add_systems(Update, update_visibility)
        // .add_systems(Update, handle_color_switching)
        .run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut graph_assets: ResMut<Assets<AnimationGraph>>,
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

    // Build the animation graph for the astronaut
    let mut astro_graph = AnimationGraph::new();
    let astro_animations = astro_graph
        .add_clips(
            [GltfAssetLabel::Animation(0).from_asset(ASTRO_PATH)]
                .into_iter()
                .map(|path| assets.load(path)),
            1.0,
            astro_graph.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let astro_graph_handle = graph_assets.add(astro_graph);

    // Build the animation graph for the coupe
    let mut coupe_path = AnimationGraph::new();
    let coupe_animations = coupe_path
        .add_clips(
            [GltfAssetLabel::Animation(0).from_asset(COUPE_PATH)]
                .into_iter()
                .map(|path| assets.load(path)),
            1.0,
            coupe_path.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let coupe_graph_handle = graph_assets.add(coupe_path);

    commands.insert_resource(Animations {
        astro_animations,
        astro_graph: astro_graph_handle,
        coupe_animations,
        coupe_graph: coupe_graph_handle,
    });
}

fn spawn(mut commands: Commands, assets: Res<AssetServer>) {
    let coupe = commands
        .spawn((
            SceneBundle {
                scene: assets.load(GltfAssetLabel::Scene(0).from_asset(COUPE_PATH)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_y(0.0))
                    .with_scale(Vec3::splat(1.0)),
                ..default()
            },
            WireframeSettings::default(),
            CoupeSceneTag,
        ))
        .id();

    let astro = commands
        .spawn((
            SceneBundle {
                scene: assets.load(GltfAssetLabel::Scene(0).from_asset(ASTRO_PATH)),
                transform: Transform::from_xyz(0.0, -1.2, 0.0)
                    .with_rotation(Quat::from_rotation_y(0.0))
                    .with_scale(Vec3::splat(1.)),
                ..default()
            },
            WireframeSettings::default(),
            AstroSceneTag,
        ))
        .id();

    let torus = commands
        .spawn((
            SceneBundle {
                scene: assets.load(GltfAssetLabel::Scene(0).from_asset(TORUS_PATH)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_y(0.0))
                    .with_scale(Vec3::splat(1.)),
                ..default()
            },
            WireframeSettings::default(),
            // ColorBuffer::default(),
        ))
        .id();
}

#[derive(Component)]
struct WheelRotator {
    rotation_speed: f32,
}

// fn rotate_wheels(time: Res<Time>, mut query: Query<(&WheelRotator, &mut Transform)>) {
//     for (wheel, mut transform) in query.iter_mut() {
//         transform.rotate_x(wheel.rotation_speed * time.delta_seconds());
//     }
// }

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
    mut wf: Query<&mut WireframeSettings>,
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

        let mut wfs;

        if wf.get(event.parent).is_err() {
            continue;
        } else {
            wfs = wf.get_mut(event.parent).unwrap();
        }

        // Out of laziness I iterate through the whole scene until I find the scene level extra which contains a json dictionary
        // that encodes the line lists generated in blender, with the index of the mesh as the key
        // there will only be one of these, so once it finds it, it parses it and breaks the loop
        // TODO: better way to locate the scene level extra

        let mut line_list_data_from_blender: Option<HashMap<String, JsonLineList>> = None;

        for e in children.iter_descendants(event.parent) {
            if let Ok(loaded_scene_extras) = scene_extras.get(e) {
                if let Some(parsed) = parse_gltf_extra_json(&loaded_scene_extras.value) {
                    line_list_data_from_blender = Some(parsed);
                    break; // you only need to parse this once
                }
            }
        }

        // Iterate through each mesh and apply the wireframe post processing
        // If scene extras were found, it will generate the line lists according to the json dictionary
        // Otherwise it will generate a line list for every edge of the mesh

        for entity in children.iter_descendants(event.parent) {
            if let Ok((mesh_handle, parent)) = mesh.get(entity) {
                if let Some(mesh) = mesh_assets.get_mut(mesh_handle) {
                    commands.entity(entity).remove::<Handle<StandardMaterial>>();

                    let smoothed_normals: Vec<[f32; 3]> = get_smoothed_normals(mesh).unwrap();
                    // invert_normals(&mut smoothed_normals);
                    mesh.insert_attribute(ATTRIBUTE_SMOOTHED_NORMAL, smoothed_normals);
                    mesh.duplicate_vertices();
                    mesh.compute_flat_normals();

                    // Check for Vertex_Color attribute
                    if !mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some() {
                        // If Vertex_Color is not present, add it with a constant color
                        let vertex_count = mesh.count_vertices();
                        let constant_color = [1.0, 0.0, 1.0, 1.0]; // White color, adjust as needed
                        let colors: Vec<[f32; 4]> = vec![constant_color; vertex_count];
                        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                    }

                    // Add FillMaterial component
                    let fill_material_handle = fill_materials.add(FillMaterial {
                        color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                        displacement: 0.0,
                        shininess: 200.0,
                        specular_strength: 1.0,
                        vertex_color_mode: 1,
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
                            line_list = mesh.mesh_to_line_list_from_json(p);
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

// from the example
fn play_animation_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parent_query: Query<&Parent>,
    astro_scenes: Query<Entity, With<AstroSceneTag>>,
    coupe_scenes: Query<Entity, With<CoupeSceneTag>>,
) {
    for (entity, mut player) in &mut players {
        /*
        // Find the top-level parent that is tagged with AstroScene or CoupeScene
        let mut current_entity = entity;
        while let Ok(parent) = parent_query.get(current_entity) {
            current_entity = parent.get();
        }
        */

        /*
        because i cheated and used chat gpt and don't understand, the below is a more verbose version of the above
        the purpose is to identify which scene we are in denoted by the AstroScene and CoupeScene components

        TODO: generic approach to this
        */

        // Start with the current entity, which has an AnimationPlayer
        let mut current_entity = entity;

        // Traverse up the hierarchy to find the top-level parent
        loop {
            // Attempt to find the Parent component of the current entity
            match parent_query.get(current_entity) {
                Ok(parent) => {
                    // If the current entity has a parent, update current_entity to the parent
                    current_entity = parent.get();
                }
                Err(_) => {
                    // If the current entity does not have a parent, break the loop
                    break;
                }
            }
        }

        if astro_scenes.get(current_entity).is_ok() {
            println!("astro scene");
            let mut transitions = AnimationTransitions::new();
            transitions
                .play(&mut player, animations.astro_animations[0], Duration::ZERO)
                .repeat();
            commands
                .entity(entity)
                .insert(animations.astro_graph.clone())
                .insert(transitions);
        } else if coupe_scenes.get(current_entity).is_ok() {
            println!("coupe scene");
            let mut transitions = AnimationTransitions::new();
            transitions
                .play(&mut player, animations.coupe_animations[0], Duration::ZERO)
                .repeat();
            commands
                .entity(entity)
                .insert(animations.coupe_graph.clone())
                .insert(transitions);
        }
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
    mut visible_model: ResMut<VisibleModel>,
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
        ui.add(egui::Slider::new(&mut shader_settings.brightness, 0.0..=30.0).text("Brightness"));
        ui.separator();
        ui.heading("Visible");
        ui.radio_value(&mut *visible_model, VisibleModel::Coupe, "Coupe");
        ui.radio_value(&mut *visible_model, VisibleModel::Astro, "Astro");
        ui.radio_value(&mut *visible_model, VisibleModel::Torus, "Torus");
        ui.radio_value(&mut *visible_model, VisibleModel::Sphere, "Sphere");
        ui.separator();
        ui.heading("Color Source");

        // use material color
        ui.radio_value(
            &mut shader_settings.vertex_color_mode,
            0,
            "Use Material Color",
        );

        // use vertex color
        ui.radio_value(
            &mut shader_settings.vertex_color_mode,
            1,
            "Use Vertex Color",
        );

        // use alt color
        ui.radio_value(&mut shader_settings.vertex_color_mode, 2, "Use Alt Color");

        // user color
        ui.separator();
        ui.heading("Color");

        let mut color = shader_settings.color.to_linear().to_f32_array();
        if ui.color_edit_button_rgba_unmultiplied(&mut color).changed() {
            shader_settings.color = Color::rgba(color[0], color[1], color[2], color[3]);
        }
    });

    // Update all OutlineMaterial instances
    // TODO: only update if changed
    for material_handle in outline_materials.iter() {
        // println!("b");
        if let Some(material) = outline_materials_assets.get_mut(material_handle) {
            // println!("c");
            material.outline_width = shader_settings.outline_width;
            material.brightness = shader_settings.brightness;
            material.vertex_color_mode = shader_settings.vertex_color_mode;
            material.color = shader_settings.color.to_linear().to_vec4();
        }
    }

    // Update all LineMaterial instances
    // TODO: only update if changed
    for material_handle in line_materials.iter() {
        // println!("d");
        if let Some(material) = line_materials_assets.get_mut(material_handle) {
            // println!("e");
            material.displacement = shader_settings.wireframe_displacement;
            material.brightness = shader_settings.brightness;
            material.vertex_color_mode = shader_settings.vertex_color_mode;
            material.color = shader_settings.color.to_linear().to_vec4();
        }
    }

    // Update all FillMaterial instances
    // TODO: only update if changed
    for material_handle in fill_materials.iter() {
        if let Some(material) = fill_materials_assets.get_mut(material_handle) {
            material.displacement = shader_settings.fill_displacement;
            material.shininess = shader_settings.fill_shininess;
            material.specular_strength = shader_settings.fill_specular_strength;
            material.vertex_color_mode = shader_settings.vertex_color_mode;
            material.color = shader_settings.color.to_linear().to_vec4();
        }
    }
}

fn update_visibility(
    visible_model: Res<VisibleModel>,
    mut coupe_query: Query<
        &mut Visibility,
        (
            With<CoupeSceneTag>,
            Without<AstroSceneTag>,
            Without<TorusSceneTag>,
            Without<SphereSceneTag>,
        ),
    >,
    mut astro_query: Query<
        &mut Visibility,
        (
            With<AstroSceneTag>,
            Without<CoupeSceneTag>,
            Without<TorusSceneTag>,
            Without<SphereSceneTag>,
        ),
    >,
    mut torus_query: Query<
        &mut Visibility,
        (
            With<TorusSceneTag>,
            Without<CoupeSceneTag>,
            Without<AstroSceneTag>,
            Without<SphereSceneTag>,
        ),
    >,
    mut sphere_query: Query<
        &mut Visibility,
        (
            With<SphereSceneTag>,
            Without<CoupeSceneTag>,
            Without<AstroSceneTag>,
            Without<TorusSceneTag>,
        ),
    >,
) {
    for mut visibility in coupe_query.iter_mut() {
        *visibility = if *visible_model == VisibleModel::Coupe {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in astro_query.iter_mut() {
        *visibility = if *visible_model == VisibleModel::Astro {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in torus_query.iter_mut() {
        *visibility = if *visible_model == VisibleModel::Torus {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in sphere_query.iter_mut() {
        *visibility = if *visible_model == VisibleModel::Sphere {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
