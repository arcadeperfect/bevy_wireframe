use bevy::{math::Vec3, prelude::Mesh, render::{mesh::{MeshVertexAttribute, VertexAttributeValues}, render_asset::RenderAssetUsages, render_resource::VertexFormat}, utils::{HashMap, HashSet}};
use rand::Rng;

use crate::line_material::{IndexLineList, LineList};


    
pub fn random_color_mesh(mesh: &mut Mesh) {
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    let mut unique_positions: Vec<([f32; 3], [f32; 4])> = Vec::new();

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
                        let color = [rng.gen(), rng.gen(), rng.gen(), 1.0];
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

    if let Some(VertexAttributeValues::Uint16x4(_)) =
        mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
    {
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

    if let Some(VertexAttributeValues::Float32x4(_)) =
        mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
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
