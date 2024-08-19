use bevy::app::DynEq;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::{MeshVertexBufferLayout, MeshVertexBufferLayoutRef};
use bevy::render::render_resource::{
    AsBindGroup, Face, PolygonMode, RenderPipelineDescriptor, ShaderRef,
    SpecializedMeshPipelineError,
};
use bitflags::bitflags;


#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct FillMaterial {
    #[uniform(0)]
    pub color: Vec4,
    #[uniform(0)]
    pub displacement: f32,
}

impl Default for FillMaterial {
    fn default() -> Self {
        Self {
            color: Vec4::new(0.0, 0.3, 0.0, 1.0),
            displacement: 1.0,
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
}