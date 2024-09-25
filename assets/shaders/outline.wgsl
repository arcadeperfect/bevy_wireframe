#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::{
    mesh_functions,
    skinning,
    morph::morph,
    view_transformations::position_world_to_clip,
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(6) joint_indices: vec4<u32>,
    @location(7) joint_weights: vec4<f32>,
#endif
#ifdef MORPH_TARGETS
    @builtin(vertex_index) index: u32,
#endif
    @location(8) smooth_normal: vec3<f32>,
    // @location(9) alt_color: vec4<f32>
};

struct VertexOutput {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_UVS_B
    @location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(4) world_tangent: vec4<f32>,
#endif
    @location(5) color: vec4<f32>,
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    @location(6) @interpolate(flat) instance_index: u32,
#endif
#ifdef VISIBILITY_RANGE_DITHER
    @location(7) @interpolate(flat) visibility_range_dither: i32,
#endif
}



// Define your OutlineMaterial structure
struct OutlineMaterial {
    flat_color: vec4<f32>,
    outline_width: f32,
    z_translate: f32,
    use_vertex_color: i32,
    brightness: f32,
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
    let displaced_position = vertex.position + vertex.smooth_normal * scaled_outline_width;

    // Transform the displaced position to world space
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(displaced_position, 1.0));

    // Transform the displaced world position to clip space
    out.position = position_world_to_clip(out.world_position.xyz);

    // Apply z-translation for outline effect
    out.position += vec4(0.0, 0.0, -material.z_translate, 0.0);
#endif

#ifdef VERTEX_COLORS
    if (material.use_vertex_color == 1) {
        out.color = vertex.color;
    // } else if (material.use_vertex_color == 2){
    //     out.color = vertex.alt_color;
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
    return in.color * material.brightness;
}
