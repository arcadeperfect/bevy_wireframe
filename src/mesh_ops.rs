use anyhow::Result;
use bevy::{
    math::Vec3,
    prelude::Mesh,
    render::{
        mesh::{Indices, MeshVertexAttribute, VertexAttributeValues},
        render_asset::RenderAssetUsages,
    },
    utils::{HashMap, HashSet},
};

use rand::Rng;
use tracing::{info, warn};

use crate::{
    load_json::{json_parse, JsonLineList},
    WireframeSettings, ATTRIBUTE_INDEX,
};

pub fn mesh_to_wireframe(mesh: &mut Mesh, settings: &WireframeSettings) -> Result<()> {
    // apply_random_vertex_colors(mesh);

    let line_list = if let Some(_) = &settings.gltf_path {
        // Only call json_parse if gltf_path is Some
        match json_parse(settings) {
            Ok(parsed_data) => mesh.mesh_to_line_list_custom(parsed_data),
            Err(e) => panic!("Failed to parse JSON: {}", e),
            // Err(e) => return Err(anyhow::anyhow!("Failed to parse JSON: {}", e)),
        }
    } else {
        info!("No JSON file provided, computing line list from mesh vertices");
        mesh.mesh_to_line_list()
    };

    let line_mesh = line_list_to_mesh(&line_list, mesh);
    *mesh = line_mesh;

    Ok(())
}

pub trait RandomizeVertexColors {
    fn randomize_vertex_colors(&mut self);
}

impl RandomizeVertexColors for Mesh {
    fn randomize_vertex_colors(&mut self) {
        apply_random_vertex_colors(self);
    }
}

fn apply_random_vertex_colors(mesh: &mut Mesh) {
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    let mut unique_positions: Vec<([f32; 3], [f32; 4])> = Vec::new();

    let mult: f32 = 20.0;

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let colors: Vec<[f32; 4]> = positions
            .iter()
            .map(|&pos| {
                match unique_positions
                    .iter()
                    .position(|&(p, _)| vec3_approx_eq(p, pos))
                {
                    Some(index) => unique_positions[index].1,
                    None => {
                        let color = [
                            rng.gen::<f32>() * mult,
                            rng.gen::<f32>() * mult,
                            rng.gen::<f32>() * mult,
                            1.0,
                        ];

                        unique_positions.push((pos, color));
                        color
                    }
                }
            })
            .collect();

        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
}

fn vec3_approx_eq(a: [f32; 3], b: [f32; 3]) -> bool {
    const EPSILON: f32 = 1e-5;
    (a[0] - b[0]).abs() < EPSILON && (a[1] - b[1]).abs() < EPSILON && (a[2] - b[2]).abs() < EPSILON
}

#[derive(Clone, Default)]
pub struct LineList {
    pub lines: Vec<(Vert, Vert)>,
}

#[derive(Debug, Clone, Default)]
pub struct Vert {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: Option<[f32; 4]>,
    pub joint_indices: Option<[u16; 4]>,
    pub joint_weights: Option<[f32; 4]>,
}

pub trait mesh_to_line_list {
    fn mesh_to_line_list(&self) -> LineList;
}

impl mesh_to_line_list for Mesh {
    fn mesh_to_line_list(&self) -> LineList {
        mesh_to_line_list(self)
    }
}

pub fn line_list_to_mesh(line_list: &LineList, mesh: &Mesh) -> Mesh {
    let mut line_mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::LineList,
        RenderAssetUsages::RENDER_WORLD,
    );

    let positions: Vec<[f32; 3]> = line_list
        .lines
        .iter()
        .flat_map(|(start, end)| vec![start.position, end.position])
        .collect();

    line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    let colors: Vec<[f32; 4]> = line_list
        .lines
        .iter()
        .flat_map(|(start, end)| vec![start.color, end.color])
        .flatten()
        .collect();

    line_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    let normal: Vec<[f32; 3]> = line_list
        .lines
        .iter()
        .flat_map(|(start, end)| vec![start.normal, end.normal])
        .collect();

    line_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normal);

    if let Some(VertexAttributeValues::Uint16x4(_)) = mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX) {
        let joint_indices: Vec<[u16; 4]> = line_list
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
        let joint_weights: Vec<[f32; 4]> = line_list
            .lines
            .iter()
            .flat_map(|(start, end)| vec![start.joint_weights, end.joint_weights])
            .flatten()
            .collect();
        line_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weights);
    }

    line_mesh
}

pub trait mesh_to_line_list_custom {
    fn mesh_to_line_list_custom(&self, data: JsonLineList) -> LineList;
}

impl mesh_to_line_list_custom for Mesh {
    fn mesh_to_line_list_custom(&self, data: JsonLineList) -> LineList {
        mesh_to_line_list_custom(self, data)
    }
}

fn mesh_to_line_list_custom(mesh: &Mesh, data: JsonLineList) -> LineList {
    let mut line_list = LineList::default();
    let mut edge_set = HashSet::new();

    if let (
        Some(VertexAttributeValues::Float32x3(positions)),
        Some(VertexAttributeValues::Float32x3(normals)),
    ) = (
        mesh.attribute(Mesh::ATTRIBUTE_POSITION),
        mesh.attribute(Mesh::ATTRIBUTE_NORMAL),
    ) {
        let colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR).and_then(|attr| {
            if let VertexAttributeValues::Float32x4(values) = attr {
                Some(values)
            } else {
                warn!("ATTRIBUTE_COLOR: invalid attribute format");
                None
            }
        });

        let joint_indices = mesh
            .attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
            .and_then(|attr| {
                if let VertexAttributeValues::Uint16x4(values) = attr {
                    Some(values)
                } else {
                    warn!("ATTRIBUTE_JOINT_INDEX: invalid attribute format");
                    None
                }
            });

        let joint_weights = mesh
            .attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
            .and_then(|attr| {
                if let VertexAttributeValues::Float32x4(values) = attr {
                    Some(values)
                } else {
                    warn!("ATTRIBUTE_JOINT_WEIGHT: invalid attribute format");
                    None
                }
            });

        let index = mesh.attribute(ATTRIBUTE_INDEX).and_then(|attr| {
            if let VertexAttributeValues::Float32(values) = attr {
                Some(values)
            } else {
                None
            }
        });

        // Create a mapping from INDEX values to vertex indices
        let mut index_to_vertex = HashMap::new();
        if let Some(index_values) = &index {
            for (vertex_index, &index_value) in index_values.iter().enumerate() {
                index_to_vertex.insert(index_value as u32, vertex_index as u32);
            }
        }

        // Process the JSON line list
        for &[index1, index2] in &data.line_list {
            if let (Some(&v1), Some(&v2)) =
                (index_to_vertex.get(&index1), index_to_vertex.get(&index2))
            {
                let edge = if v1 < v2 { (v1, v2) } else { (v2, v1) };
                if edge_set.insert(edge) {
                    let i1 = v1 as usize;
                    let i2 = v2 as usize;
                    let start = Vert {
                        position: positions[i1],
                        normal: normals[i1],
                        color: colors.map(|c| c[i1]),
                        joint_indices: joint_indices.map(|ji| ji[i1]),
                        joint_weights: joint_weights.map(|jw| jw[i1]),
                    };
                    let end = Vert {
                        position: positions[i2],
                        normal: normals[i2],
                        color: colors.map(|c| c[i2]),
                        joint_indices: joint_indices.map(|ji| ji[i2]),
                        joint_weights: joint_weights.map(|jw| jw[i2]),
                    };
                    line_list.lines.push((start, end));
                }
            } else {
                warn!("Warning: INDEX {} or {} not found in mesh", index1, index2);
            }
        }
    }
    line_list
}

fn mesh_to_line_list(mesh: &Mesh) -> Result<LineList> {
    
    let mut line_list = LineList::default();
    let mut edge_set = HashSet::new();

    if let (
        Some(VertexAttributeValues::Float32x3(positions)),
        Some(VertexAttributeValues::Float32x3(normals)),
    ) = (
        mesh.attribute(Mesh::ATTRIBUTE_POSITION),
        mesh.attribute(Mesh::ATTRIBUTE_NORMAL),
    ) {
        let colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR).and_then(|attr| {
            if let VertexAttributeValues::Float32x4(values) = attr {
                Some(values)
            } else {
                warn!("invalid attribute format");
                None
            }
        });

        let joint_indices = mesh
            .attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
            .and_then(|attr| {
                if let VertexAttributeValues::Uint16x4(values) = attr {
                    Some(values)
                } else {
                    warn!("invalid attribute format");
                    None
                }
            });

        let joint_weights = mesh
            .attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
            .and_then(|attr| {
                if let VertexAttributeValues::Float32x4(values) = attr {
                    Some(values)
                } else {
                    warn!("invalid attribute format");
                    None
                }
            });

        let mut process_triangle = |a: usize, b: usize, c: usize| {
            let mut add_edge = |v1: usize, v2: usize| {
                let edge = if v1 < v2 { (v1, v2) } else { (v2, v1) };
                if edge_set.insert(edge) {
                    let start = Vert {
                        position: positions[v1],
                        normal: normals[v1],
                        color: colors.map(|c| c[v1]),
                        joint_indices: joint_indices.map(|ji| ji[v1]),
                        joint_weights: joint_weights.map(|jw| jw[v1]),
                    };
                    let end = Vert {
                        position: positions[v2],
                        normal: normals[v2],
                        color: colors.map(|c| c[v2]),
                        joint_indices: joint_indices.map(|ji| ji[v2]),
                        joint_weights: joint_weights.map(|jw| jw[v2]),
                    };
                    line_list.lines.push((start, end));
                }
            };

            add_edge(a, b);
            add_edge(b, c);
            add_edge(c, a);
        };

        if let Some(indices) = mesh.indices() {
            match indices {
                Indices::U16(idx) => {
                    for triangle in idx.chunks(3) {
                        process_triangle(
                            triangle[0] as usize,
                            triangle[1] as usize,
                            triangle[2] as usize,
                        );
                    }
                }
                Indices::U32(idx) => {
                    for triangle in idx.chunks(3) {
                        process_triangle(
                            triangle[0] as usize,
                            triangle[1] as usize,
                            triangle[2] as usize,
                        );
                    }
                }
            }
        } else {
            // Handle non-indexed geometry
            for (i, triangle) in positions.chunks(3).enumerate() {
                if triangle.len() == 3 {
                    process_triangle(i * 3, i * 3 + 1, i * 3 + 2);
                }
            }
        }
    } else {
        // warn!("mesh missing required data");
        return Err(anyhow!("mesh missing required data"));
    }

    if line_list.lines.is_empty() {
        return Err(anyhow!("no lines generated"));
    }

    Ok(line_list)
}


pub trait SmoothNormalsNonIndexed {
    fn smooth_normals_non_indexed(&mut self);
}

impl SmoothNormalsNonIndexed for Mesh {
    fn smooth_normals_non_indexed(&mut self) {
        smooth_normals_non_indexed(self);
    }
}

fn smooth_normals_non_indexed(mesh: &mut Mesh) {
    if let (
        Some(VertexAttributeValues::Float32x3(positions)),
        Some(VertexAttributeValues::Float32x3(normals)),
    ) = (
        mesh.attribute(Mesh::ATTRIBUTE_POSITION),
        mesh.attribute(Mesh::ATTRIBUTE_NORMAL),
    ) {
        let mut normal_map: HashMap<(i32, i32, i32), Vec3> = HashMap::new();

        // Function to convert float position to integer key
        let to_key = |pos: &[f32; 3]| {
            (
                (pos[0] * 1000.0).round() as i32,
                (pos[1] * 1000.0).round() as i32,
                (pos[2] * 1000.0).round() as i32,
            )
        };

        // Sum up normals for each unique position
        for (position, normal) in positions.iter().zip(normals.iter()) {
            let key = to_key(position);
            let entry = normal_map.entry(key).or_insert(Vec3::ZERO);
            *entry += Vec3::from_array(*normal);
        }

        // Normalize the summed normals
        for normal in normal_map.values_mut() {
            *normal = normal.normalize();
        }

        // Create new normalized normals
        let new_normals: Vec<[f32; 3]> = positions
            .iter()
            .map(|pos| normal_map[&to_key(pos)].to_array())
            .collect();

        // Update the mesh with new normals
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, new_normals);
    }
}
