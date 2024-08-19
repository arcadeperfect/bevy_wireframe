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


// @vertex
// fn vertex(vertex: Vertex, @builtin(instance_index) instance_index: u32) -> VertexOutput {
   
//     var out: VertexOutput;

   
//     // Get the world_from_local matrix for this instance
//     let world_from_local = get_world_from_local(instance_index);

//      // Transform the displaced position to world space
//     out.world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    
//     // Transform the world position to clip space
//     out.clip_position = mesh_position_local_to_clip(world_from_local, vec4<f32>(vertex.position, 1.0));
    
//     // Transform the normal to world space
//     out.world_normal = mesh_normal_local_to_world(vertex.normal, instance_index);

//     // out.view_from_world = view.view_from_world;
    
//     return out;
// }

// @vertex
// fn vertex(vertex: Vertex, @builtin(instance_index) instance_index: u32) -> VertexOutput {
//     var out: VertexOutput;


//     // Get the world_from_local matrix for this instance
//     let world_from_local = get_world_from_local(instance_index);
    


//     // get camera position
//     let camera_position = view.world_position.xyz;
//     let world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    
//     let view_direction = normalize(camera_position - world_position);




//     // Displace the vertex along its normal
//     let displaced_position = vertex.position + vertex.normal * 0.01;

//     let translated_position = displaced_position + camera_position * 0.0;
    
//     // Transform the displaced position to world space
//     out.world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(translated_position, 1.0));
    
//     // Transform the world position to clip space
//     out.clip_position = mesh_position_local_to_clip(world_from_local, vec4<f32>(translated_position, 1.0));
    
//     // Transform the normal to world space
//     out.world_normal = mesh_normal_local_to_world(vertex.normal, instance_index);
    
//     return out;
// }

@vertex
fn vertex(vertex: Vertex, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Get the world_from_local matrix for this instance
    let world_from_local = get_world_from_local(instance_index);

    // Get camera position
    let camera_position = view.world_position.xyz;
    let world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    
    // Ensure world_position is vec3 for the view_direction calculation
    let world_position_xyz = world_position.xyz;
    let view_direction = normalize(camera_position - world_position_xyz);

    // Displace the vertex along its normal
    let displaced_position = vertex.position + vertex.normal * 0.01;

    let translated_position = displaced_position + camera_position * 0.01;
    
    // Transform the displaced position to world space
    out.world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(translated_position, 1.0));
    
    // Transform the world position to clip space
    out.clip_position = mesh_position_local_to_clip(world_from_local, vec4<f32>(translated_position, 1.0));
    
    // Transform the normal to world space
    out.world_normal = mesh_normal_local_to_world(vertex.normal, instance_index);
    
    return out;
}



@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
   
    

    return vec4<f32>(1.0,1.0,0.0,1.0);
}