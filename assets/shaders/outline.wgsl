// #import bevy_pbr::mesh_view_bindings
// #import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_view_bindings::view
// #import bevy_pbr::mesh_functions

// struct OutlineMaterial {
//     flat_color: vec4<f32>,
//     outline_width: f32,
//     z_translate: f32,
//     use_vertex_color: i32,
// };

// @group(2) @binding(0)
// var<uniform> material: OutlineMaterial;

// struct Vertex {
//     @location(0) position: vec3<f32>,
//     @location(1) normal: vec3<f32>,
//     #ifdef VERTEX_COLOR_AVAILABLE
//     @location(5) color: vec4<f32>,
//     #endif
// };

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) world_position: vec4<f32>,
//     @location(1) world_normal: vec3<f32>,
//     @location(2) view_position: vec4<f32>,
//     @location(3) color: vec4<f32>,
// };

// @vertex
// fn vertex(vertex: Vertex, @builtin(instance_index) instance_index: u32) -> VertexOutput {
//     var out: VertexOutput;

//     // Get the transformation from local to world space
//     let world_from_local = mesh_functions::get_world_from_local(instance_index);

//     // Compute the world position of the vertex
//     let world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));

//     // Calculate the camera position in world space
//     let camera_position = view.world_position.xyz;

//     // Compute the distance from the camera to the vertex
//     let distance_to_camera = length(world_position.xyz - camera_position);

//     // Scale the outline width based on the distance to maintain visual consistency
//     let scaled_outline_width = material.outline_width * distance_to_camera * 0.01;

//     // Displace the vertex along its normal by the scaled outline width
//     let displaced_position = vertex.position + vertex.normal * scaled_outline_width;

//     // Transform the displaced position to world space
//     out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(displaced_position, 1.0));

//     // Transform the world position to clip space
//     out.clip_position = mesh_functions::mesh_position_local_to_clip(world_from_local, vec4<f32>(displaced_position, 1.0));

//     // Transform the normal to world space
//     out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, instance_index);

//     // Adjust the clip position to apply z-translation
//     out.clip_position += vec4(0.0, 0.0, -material.z_translate, 0.0);

//     // Set the vertex color

//     #ifdef VERTEX_COLOR_AVAILABLE
//         if(material.use_vertex_color > 0){
//             out.color = vertex.color;
//         } else {
//             out.color = material.flat_color;
//         }
//     #else
//         out.color = material.flat_color;
//     #endif

//     return out;
// }

// @fragment
// fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {

//     return in.color;
// }

#import bevy_pbr::{
    mesh_functions,
    skinning,
    morph::morph,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
}

// Define your OutlineMaterial structure
struct OutlineMaterial {
    flat_color: vec4<f32>,
    outline_width: f32,
    z_translate: f32,
    use_vertex_color: i32,
};

@group(2) @binding(0)
var<uniform> material: OutlineMaterial;

#ifdef MORPH_TARGETS
fn morph_vertex(vertex_in: Vertex) -> Vertex {
    var vertex = vertex_in;
    let weight_count = bevy_pbr::morph::layer_count();
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::morph::weight_at(i);
        if (weight == 0.0) {
            continue;
        }
        vertex.position += weight * morph(vertex.index, bevy_pbr::morph::position_offset, i);
#ifdef VERTEX_NORMALS
        vertex.normal += weight * morph(vertex.index, bevy_pbr::morph::normal_offset, i);
#endif
#ifdef VERTEX_TANGENTS
        vertex.tangent += vec4(weight * morph(vertex.index, bevy_pbr::morph::tangent_offset, i), 0.0);
#endif
    }
    return vertex;
}
#endif

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph);
#else
    var vertex = vertex_no_morph;
#endif

#ifdef SKINNED
    var world_from_local = skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
#else
    var world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(world_from_local, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_POSITIONS
    // Original world position before outline displacement
    let original_world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));

    // Calculate the camera position in world space
    let camera_position = view.world_position.xyz;

    // Compute the distance from the camera to the vertex
    let distance_to_camera = length(original_world_position.xyz - camera_position);

    // Scale the outline width based on the distance to maintain visual consistency
    let scaled_outline_width = material.outline_width * distance_to_camera * 0.01;

    // Displace the vertex along its normal by the scaled outline width
    let displaced_position = vertex.position + vertex.normal * scaled_outline_width;

    // Transform the displaced position to world space
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(displaced_position, 1.0));

    // Transform the displaced world position to clip space
    out.position = position_world_to_clip(out.world_position.xyz);

    // Apply z-translation for outline effect
    out.position += vec4(0.0, 0.0, -material.z_translate, 0.0);
#endif

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        vertex_no_morph.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    if (material.use_vertex_color > 0) {
        out.color = vertex.color;
    } else {
        out.color = material.flat_color;
    }
#else
    out.color = material.flat_color;
#endif

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
