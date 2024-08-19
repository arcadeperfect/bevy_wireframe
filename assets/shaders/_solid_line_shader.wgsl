#import bevy_pbr::forward_io::VertexOutput

struct LineMaterial {
    displacement: f32,
};

@group(2) @binding(0) var<uniform> material: LineMaterial;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
};

@vertex
fn vertex(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Use the original position for the wireframe
    let wire_position = position;
    // Displace the position along its normal for the solid mesh
    let solid_position = position + normal * material.displacement;
    
    // Output both positions
    out.clip_position = bevy_pbr::mesh_functions::mesh_position_local_to_clip(model, vec4<f32>(wire_position, 1.0));
    out.world_position = bevy_pbr::mesh_functions::mesh_position_local_to_world(model, vec4<f32>(solid_position, 1.0));
    out.world_normal = bevy_pbr::mesh_functions::mesh_normal_local_to_world(normal);
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Hard-coded colors
    let solid_color = vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black
    let wire_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);  // White

    // Hard-coded line width
    let line_width = 1.0;

    // Blend between solid and wire based on line width
    let t = smoothstep(0.0, line_width, length(dpdx(in.world_position.xy)) + length(dpdy(in.world_position.xy)));
    return mix(solid_color, wire_color, t);
}