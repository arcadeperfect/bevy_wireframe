use bevy::{
    math::Vec3,
    prelude::Mesh,
    render::{
        mesh::{Indices, MeshVertexAttribute, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::VertexFormat,
    },
    utils::{HashMap, HashSet},
};
use rand::Rng;

use crate::{load_json::jparse, ATTRIBUTE_CUSTOM, ATTRIBUTE_INDEX};

pub fn random_color_mesh(mesh: &mut Mesh) {
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

pub fn smooth_normals(mesh: &mut Mesh) {
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

pub fn generate_edge_line_list(mesh: &Mesh, use_custom_attr: bool) -> LineList {
    let mut line_list = LineList::default();
    let mut edge_set = HashSet::new();

    let json_line_list = jparse();

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
                println!("invalid attribute format");
                None
            }
        });

        let joint_indices = mesh
            .attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
            .and_then(|attr| {
                if let VertexAttributeValues::Uint16x4(values) = attr {
                    Some(values)
                } else {
                    println!("invalid attribute format");
                    None
                }
            });

        let joint_weights = mesh
            .attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
            .and_then(|attr| {
                if let VertexAttributeValues::Float32x4(values) = attr {
                    Some(values)
                } else {
                    println!("invalid attribute format");
                    None
                }
            });

        let index = mesh.attribute(ATTRIBUTE_INDEX).and_then(|attr| {
            if let VertexAttributeValues::Float32(values) = attr {
                println!("found index attr");
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

        if use_custom_attr {
            // Process the JSON line list
            for &[index1, index2] in &json_line_list.line_list {
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
                    println!("Warning: INDEX {} or {} not found in mesh", index1, index2);
                }
            }
        } else {
            return generate_edge_line_list_old(mesh);
        }
    }

    line_list
}

fn generate_edge_line_list_old(mesh: &Mesh) -> LineList {
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
                println!("invalid attribute format");
                None
            }
        });

        let joint_indices = mesh
            .attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
            .and_then(|attr| {
                if let VertexAttributeValues::Uint16x4(values) = attr {
                    Some(values)
                } else {
                    println!("invalid attribute format");
                    None
                }
            });

        let joint_weights = mesh
            .attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
            .and_then(|attr| {
                if let VertexAttributeValues::Float32x4(values) = attr {
                    Some(values)
                } else {
                    println!("invalid attribute format");
                    None
                }
            });

        let custom = mesh.attribute(ATTRIBUTE_CUSTOM).and_then(|attr| {
            if let VertexAttributeValues::Float32(values) = attr {
                Some(values)
            } else {
                None
            }
        });

        let mut process_triangle = |a: u32, b: u32, c: u32| {
            let mut add_edge = |v1: u32, v2: u32| {

                if (true) {
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
                            triangle[0] as u32,
                            triangle[1] as u32,
                            triangle[2] as u32,
                        );
                    }
                }
                Indices::U32(idx) => {
                    for triangle in idx.chunks(3) {
                        process_triangle(
                            triangle[0] as u32,
                            triangle[1] as u32,
                            triangle[2] as u32,
                        );
                    }
                }
            }
        }
    } else {
        println!("mesh missing required data")
    }

    line_list
}