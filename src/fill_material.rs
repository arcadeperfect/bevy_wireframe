use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError
};

// use crate::{ATTRIBUTE_ALT_COLOR};
use crate::{ATTRIBUTE_SMOOTHED_NORMAL};
// use bitflags::bitflags;


#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct FillMaterial {
    #[uniform(0)]
    pub color: Vec4,
    #[uniform(0)]
    pub displacement: f32,
    #[uniform(0)]
    pub shininess: f32,
    #[uniform(0)]
    pub specular_strength: f32,
    #[uniform(0)]
    pub vertex_color_mode: i32,
}

impl Default for FillMaterial {
    fn default() -> Self {
        Self {
            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            displacement: 0.1,
            shininess: 200.0,
            specular_strength: 1.0,
            vertex_color_mode: 0,
        }
    }
}

impl Material for FillMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fill.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/fill.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let mut attributes = vec![
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(5),
            ATTRIBUTE_SMOOTHED_NORMAL.at_shader_location(8),
            // ATTRIBUTE_ALT_COLOR.at_shader_location(9),
        ];

        if layout.0.contains(Mesh::ATTRIBUTE_JOINT_INDEX)
            && layout.0.contains(Mesh::ATTRIBUTE_JOINT_WEIGHT)
        {
            attributes.push(Mesh::ATTRIBUTE_JOINT_INDEX.at_shader_location(6));
            attributes.push(Mesh::ATTRIBUTE_JOINT_WEIGHT.at_shader_location(7));
        }

        let vertex_layout = layout.0.get_layout(&attributes)?;

        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}