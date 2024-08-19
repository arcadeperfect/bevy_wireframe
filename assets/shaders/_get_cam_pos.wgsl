#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_view_bindings::view

#import bevy_pbr::mesh_functions::{
    get_world_from_local, 
    mesh_position_local_to_clip, 
    mesh_position_local_to_world,
    mesh_normal_local_to_world
    }


struct OutlineMaterial{
    color: vec4<f32>,
};


struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) view_position: vec4<f32>,
    // @location(3) view_from_world: mat4x4<f32>, // Add this line

};



@vertex
fn vertex(vertex: Vertex, @builtin(instance_index) instance_index: u32) -> VertexOutput {
   
    var out: VertexOutput;

   
    // Get the world_from_local matrix for this instance
    let world_from_local = get_world_from_local(instance_index);

     // Transform the displaced position to world space
    out.world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    
    // Transform the world position to clip space
    out.clip_position = mesh_position_local_to_clip(world_from_local, vec4<f32>(vertex.position, 1.0));
    
    // Transform the normal to world space
    out.world_normal = mesh_normal_local_to_world(vertex.normal, instance_index);

    // out.view_from_world = view.view_from_world;
    
    return out;
}


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get the world position of the current fragment
    let world_position = in.world_position.xyz;
    
    // Calculate the direction from the fragment to the camera
    let camera_position = view.world_position.xyz;
    let camera_direction = normalize(camera_position - world_position);
    
    // Use camera_direction in your lighting calculations
    // ...

    // Return your final color
    // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    
    return vec4<f32>(camera_direction.x, camera_direction.y, 0.0, 1.0);

    }