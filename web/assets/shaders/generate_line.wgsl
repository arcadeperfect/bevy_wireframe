struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Hardcoded line start and end positions
    let start_position = vec3<f32>(-0.5, 0.0, 0.0);
    let end_position = vec3<f32>(0.5, 0.0, 0.0);
    
    // Hardcoded color (red)
    let line_color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    
    // Determine whether this is the start or end vertex of the line
    if (vertex_index == 0u) {
        out.clip_position = vec4<f32>(start_position, 1.0);
    } else {
        out.clip_position = vec4<f32>(end_position, 1.0);
    }
    
    out.color = line_color;
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}