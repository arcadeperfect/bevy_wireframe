// Custom vertex shader that offsets the verteces by a random amount 

#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip, mesh_position_local_to_world}


struct Input {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: Input;


struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ (state >> 16u);
    state = state * 2654435769u;
    state = state ^ (state >> 16u);
    state = state * 2654435769u;
    return state;
}

fn random(value: f32) -> f32 {
    return f32(hash(u32(value * 1000.0))) / 4294967295.0;
}




@vertex
fn vertex(vertex: Vertex, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    let seed = f32(instance_index) + vertex.position.x * 10.0 + vertex.position.y * 100.0 + vertex.position.z * 1000.0;


    let offset = (random(f32(seed)) * 2.0 - 1.0) * 0.1; 

    let world_from_local = get_world_from_local(instance_index);
    let offset_position = vec4<f32>(vertex.position + offset, 1.0);
    
    out.world_position = mesh_position_local_to_world(world_from_local, offset_position);
    out.clip_position = mesh_position_local_to_clip(world_from_local, offset_position);
    
    return out;
}


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return material.color;
}


