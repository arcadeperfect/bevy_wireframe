// use bitflags::bitflags;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};

// use crate::{ATTRIBUTE_ALT_COLOR};
use crate::{ATTRIBUTE_SMOOTHED_NORMAL};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct OutlineMaterial {
    #[uniform(0)]
    pub color: Vec4,
    #[uniform(0)]
    pub outline_width: f32,
    #[uniform(0)]
    pub z_translate: f32,
    #[uniform(0)]
    pub vertex_color_mode: i32,
    #[uniform(0)]
    pub brightness: f32,
}

impl Material for OutlineMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/outline.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/outline.wgsl".into()
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

impl Default for OutlineMaterial {
    fn default() -> Self {
        Self {
            color: Vec4::new(0.6, 1.0, 0.6, 1.0),
            outline_width: 0.0,
            z_translate: 0.1,
            vertex_color_mode: 0,
            brightness: 15.0,
        }
    }
}
