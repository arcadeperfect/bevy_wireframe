// #import bevy_pbr::{
//     forward_io::{Vertex, VertexOutput},
// }

struct Vertex {
    @location(1) index: u32,
    @location(5) color: vec4<f32>,

};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(5) color: vec4<f32>,

};

struct LineMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: LineMaterial;

// This buffer will hold the positions of all vertices in the original mesh
@group(1) @binding(0) var<storage, read> positions: array<vec3<f32>>;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    // Retrieve the position from the positions buffer using the vertex index
    let position = positions[vertex.index];
    
    out.clip_position = vec4<f32>(position, 1.0);
    // out.color = vertex.color;
    
    // #ifdef VERTEX_COLORS
    // out.color = vertex.color;
    // #endif

    #ifdef VERTEX_COLORS
    out.color = vertex.color;
    #endif

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // #ifdef VERTEX_COLORS
    // return in.color;
    // #else
    return in.color;
    // #endif
    
}