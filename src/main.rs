use bevy::gltf::GltfExtras;
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

// mod window_resize_plugin;
// use window_resize_plugin::WindowResizePlugin;

use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use fill_material::FillMaterial;
use line_material::LineMaterial;
use mesh_ops::{mesh_to_wireframe, RandomizeVertexColors, SmoothNormalsNonIndexed};
use outline_material::OutlineMaterial;
use std::time::Duration;

mod camera_plugin;
mod fill_material;
mod line_material;
mod load_json;
mod mesh_ops;
mod outline_material;

// const PATH: &str = "astro/scene.gltf";
const ASTROPATH: &str = "astro_custom/scene.gltf";
// const FOXPATH: &str = "fox.glb";
// const PATH: &str = "sphere_flat.gltf";
// const PATH: &str = "sphere.gltf";
// const PATH: &str = "torus.gltf";
const TORUSPATH: &str = "temp/torus_custom_property.gltf";

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

const ATTRIBUTE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("Index", 1237464976, VertexFormat::Float32);

fn main() {
    App::new()
        // .add_plugins(WindowResizePlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(ShaderSettings::default())
        .add_plugins(
            DefaultPlugins
                .set(GltfPlugin::default().add_custom_vertex_attribute("INDEX", ATTRIBUTE_INDEX)),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(MaterialPlugin::<FillMaterial>::default())
        .add_plugins(MaterialPlugin::<OutlineMaterial>::default())
        .add_plugins(MaterialPlugin::<LineMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, play_animation_once_loaded.before(animate_targets))
        .add_systems(Update, process_scene)
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
            [GltfAssetLabel::Animation(0).from_asset(ASTROPATH)]
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

    let astro = commands
        .spawn((
            SceneBundle {
                scene: assets.load(GltfAssetLabel::Scene(0).from_asset(ASTROPATH)),
                transform: Transform::from_xyz(0.0, -1.2, 0.0)
                    .with_rotation(Quat::from_rotation_y(0.0))
                    .with_scale(Vec3::splat(1.)),
                ..default()
            },
            WireframeSettings {
                // gltf_path: None,
                // gltf_path: Some(String::from(ASTROPATH)),

            },
        ))
        .id();

    // let torus = commands
    //     .spawn((
    //         SceneBundle {
    //             scene: assets.load(GltfAssetLabel::Scene(0).from_asset(TORUSPATH)),
    //             transform: Transform::from_xyz(0.0, 0.0, 0.0)
    //                 .with_rotation(Quat::from_rotation_y(0.0))
    //                 .with_scale(Vec3::splat(1.)),
    //             ..default()
    //         },
    //         WireframeSettings {
            
    //         },
    //     ))
    //     .id();

    // let fox = commands
    //     .spawn((
    //         SceneBundle {
    //             scene: assets.load(GltfAssetLabel::Scene(0).from_asset(FOXPATH)),
    //             ..default()
    //         },
    //         WireframeSettings {
    //             gltf_path: None,
    //             // gltf_path: Some(String::from(FOXPATH)),
    //         },
    //     ))
    //     .id();
}



fn process_scene(
    mut commands: Commands,
    mut events: EventReader<SceneInstanceReady>,
    children: Query<&Children>,
    meshes: Query<(Entity, &Handle<Mesh>)>,
    skinned_meshes: Query<&SkinnedMesh>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    mut fill_materials: ResMut<Assets<FillMaterial>>, // Add FillMaterial resource
    mut outline_materials: ResMut<Assets<OutlineMaterial>>, // Add FillMaterial resource
    shader_settings: Res<ShaderSettings>,
    processable_scenes: Query<&WireframeSettings>,
    gltf_extras: Query<(Entity, &GltfExtras)>, // Modified this line
) {
    for event in events.read() {
        if !processable_scenes.contains(event.parent) {
            continue;
        }

        // Print extras for the parent entity
        if let Ok((_, extras)) = gltf_extras.get(event.parent) {
            println!("Parent Extras: {:?}", extras);
        }
        
        // if event.parent == my_scene_entity.0 {
        for entity in children.iter_descendants(event.parent) {
            if let Ok((_, extras)) = gltf_extras.get(entity) {
                println!("Child Entity Extras: {:?}", extras);
            }

            if let (Ok((entity, mesh_handle)), Ok(wireframe_settings)) =
                (meshes.get(entity), processable_scenes.get(event.parent))
            {
                if let Some(flat_mesh) = mesh_assets.get_mut(mesh_handle) {
                    commands.entity(entity).remove::<Handle<StandardMaterial>>();
                    flat_mesh.randomize_vertex_colors();

                    let mut smooth_mesh = flat_mesh.clone();
                    // smooth_mesh.compute_smooth_normals();
                    smooth_mesh.smooth_normals_non_indexed();
                    flat_mesh.duplicate_vertices();
                    flat_mesh.compute_flat_normals();

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

                    match mesh_to_wireframe(&mut smooth_mesh, &wireframe_settings) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Error: {:?}", e);
                        }
                    }
                    // mesh_to_wireframe(&mut smooth_mesh, &wireframe_settings);

                    let new_mesh_handle = mesh_assets.add(smooth_mesh);
                    let skinned_mesh = skinned_meshes.get(entity).cloned();

                    let bundle = MaterialMeshBundle {
                        mesh: new_mesh_handle,
                        material: line_materials.add(LineMaterial {
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
