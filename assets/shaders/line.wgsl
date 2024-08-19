#import bevy_pbr::forward_io::VertexOutput

struct LineShader {
    color: vec4<f32>,
};

@group(2) @binding(0) var<uniform> material: LineShader;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // return material.color;
    return in.color;
    // return vec4<f32>(0.0,0.0,1.0,0.0);
}
