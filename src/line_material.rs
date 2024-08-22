use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{
            AsBindGroup, ShaderRef,
        },
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LineMaterial {
    #[uniform(0)]
    pub color: Vec4,
    #[uniform(0)]
    pub displacement: f32,
}


impl Default for LineMaterial {
    fn default() -> Self {
        Self {
            color: Vec4::new(1.0, 0.3, 1.0, 1.0),
            displacement: 0.0,
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
}


