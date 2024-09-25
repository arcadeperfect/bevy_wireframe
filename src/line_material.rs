use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey}, prelude::*, reflect::TypePath, render::{mesh::MeshVertexBufferLayoutRef, render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError
        }}
};

// use crate::{ATTRIBUTE_ALT_COLOR};
use crate::{ATTRIBUTE_SMOOTHED_NORMAL};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LineMaterial {
    #[uniform(0)]
    pub color: Vec4,
    #[uniform(0)]
    pub displacement: f32,
    #[uniform(0)]
    pub vertex_color_mode: i32,
    #[uniform(0)]
    pub brightness: f32,
}


impl Default for LineMaterial {
    fn default() -> Self {
        Self {
            color: Vec4::new(1.0, 0.3, 0.0, 1.0),
            displacement: 0.0,
            vertex_color_mode: 0,
            brightness: 15.0,
        }
    }
}

impl Material for LineMaterial {

    fn vertex_shader() -> ShaderRef {
        "shaders/line.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/line.wgsl".into()
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
            // ATTRIBUTE_SMOOTHED_NORMAL.at_shader_location(8),
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


