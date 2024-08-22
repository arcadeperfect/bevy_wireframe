// use bitflags::bitflags;
use bevy::app::DynEq;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::{MeshVertexBufferLayout, MeshVertexBufferLayoutRef};
use bevy::render::render_resource::{
    AsBindGroup, Face, PolygonMode, RenderPipelineDescriptor, ShaderRef,
    SpecializedMeshPipelineError,
};

// bitflags! {
//     /// Bitflags representing the configuration for the `OutlineMaterial`.
//     #[derive(Clone, Copy, PartialEq, Eq, Hash)]
//     pub struct OutlineMaterialKey: u64 {
//         const USE_VERTEX_COLOR = 0x0001;
//     }
// }



#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct OutlineMaterial {
    #[uniform(0)]
    pub flat_color: Vec4,
    #[uniform(0)]
    pub outline_width: f32,
    #[uniform(0)]
    pub z_translate: f32,
    #[uniform(0)]
    pub use_vertex_color: i32,
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

        let defs = &mut descriptor.vertex.shader_defs;

        let mesh_layout = &layout.0;
        if mesh_layout.contains(Mesh::ATTRIBUTE_COLOR) {
            defs.push("VERTEX_COLOR_AVAILABLE".into());
        }


        // defs.push("VERTEX_COLOR_AVAILABLE".into());

        Ok(())
    }
}

impl Default for OutlineMaterial {
    fn default() -> Self {
        Self {
            flat_color: Vec4::new(0.6, 1.0, 0.6, 1.0),
            outline_width: 0.0,
            z_translate: 0.1,
            use_vertex_color: 1,
        }
    }
}