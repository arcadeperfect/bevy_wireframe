struct Vertex {
    @location(1) index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

// This buffer will hold the positions of all vertices in the original mesh
@group(1) @binding(0) var<storage, read> positions: array<vec3<f32>>;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    // Retrieve the position from the positions buffer using the vertex index
    let position = positions[vertex.index];
    
    out.clip_position = vec4<f32>(position, 1.0);
    // out.color = vertex.color;
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0,0.0,1.0,1.0);
}