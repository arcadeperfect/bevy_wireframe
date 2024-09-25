#import bevy_pbr::{
    mesh_functions,
    skinning,
    morph::morph,
    // forward_io::{Vertex},
    view_transformations::position_world_to_clip,
    view_transformations::position_view_to_world,

    mesh_view_bindings::view
}

struct VertexOutput {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
// #ifdef VERTEX_UVS_A
//     @location(2) uv: vec2<f32>,
// #endif
// #ifdef VERTEX_UVS_B
//     @location(3) uv_b: vec2<f32>,
// #endif
// #ifdef VERTEX_TANGENTS
//     @location(4) world_tangent: vec4<f32>,
// #endif
    @location(5) color: vec4<f32>,
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    @location(6) @interpolate(flat) instance_index: u32,
#endif
#ifdef VISIBILITY_RANGE_DITHER
    @location(7) @interpolate(flat) visibility_range_dither: i32,
#endif
    // @location(9) alt_color: vec4<f32>

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



struct FillMaterial {
    color: vec4<f32>,
    displacement: f32,
    shininess: f32,
    specular_strength: f32,
    use_vertex_color: i32,
};

@group(2) @binding(0)
var<uniform> material: FillMaterial;

#ifdef MORPH_TARGETS
fn morph_vertex(vertex_in: Vertex) -> Vertex {
    var vertex = vertex_in;
    let weight_count = bevy_pbr::morph::layer_count();
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::morph::weight_at(i);
        if weight == 0.0 {
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
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(world_from_local, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_POSITIONS
    // let displaced_position = vertex.position + vertex.normal * material.displacement;
    // let displaced_position = vertex.position + vertex.normal * -10;

    let world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.world_position = world_position;
    out.position = position_world_to_clip(out.world_position.xyz + vertex.normal * -material.displacement); 
#endif

// #ifdef VERTEX_UVS_A
//     out.uv = vertex.uv;
// #endif
// #ifdef VERTEX_UVS_B
//     out.uv_b = vertex.uv_b;
// #endif

// #ifdef VERTEX_TANGENTS
//     out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
//         world_from_local,
//         vertex.tangent,
//         // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
//         // See https://github.com/gfx-rs/naga/issues/2416
//         vertex_no_morph.instance_index
//     );
// #endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    out.instance_index = vertex_no_morph.instance_index;
#endif

// #ifdef VISIBILITY_RANGE_DITHER
//     out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
//         vertex_no_morph.instance_index, world_from_local[3]);
// #endif

    // out.alt_color = vertex.alt_color;

    return out;
}

// @fragment
// fn fragment(
//     mesh: VertexOutput,
// ) -> @location(0) vec4<f32> {

//     return material.color;
// }
@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    

    let camera_position = position_view_to_world(vec3(0.0, 0.0, 0.0));
    
    // Calculate light direction from camera to fragment
    let light_dir = normalize(camera_position - mesh.world_position.xyz);
    
    // Hard-coded light color (white)
    let light_color = vec3(1.0, 1.0, 1.0);
    
    // Calculate view direction (from fragment to camera)
    let view_dir = light_dir;  // Since light is coming from camera, view_dir is opposite of light_dir
    
    let normal = normalize(mesh.world_normal);
    
    let diffuse_factor = max(dot(normal, light_dir), 0.0);
    
    // Calculate the halfway vector for Blinn-Phong
    let halfway_dir = normalize(light_dir + view_dir);
    
    // Calculate the specular factor
    let specular_factor = pow(max(dot(normal, halfway_dir), 0.0), material.shininess);
    
    // Calculate the diffuse color
    let diffuse_color = light_color * diffuse_factor;
    
    // Calculate the specular color
    let specular_color = light_color * specular_factor * material.specular_strength;
    
    // Combine the material color with the lighting
    // let lighting_color = material.color.rgb * (diffuse_color + 0.2) + specular_color;
    let lighting_color = vec3<f32>(0.0,0.0,0.0) * (diffuse_color + 0.2) + specular_color;
    
    // let final_color = vec3<f32>(1.0,0.0,1.0);
    
    // Multiply with vertex color
    if (material.use_vertex_color == 1) {
        return vec4<f32>(lighting_color * mesh.color.rgb, 1.0);
    // }else if (material.use_vertex_color == 2) {
    //     return vec4<f32>(lighting_color * mesh.alt_color.rgb, 1.0);
    }else{
        return vec4<f32>(lighting_color * material.color.rgb, 1.0);
    }
    
    // let final_color = lighting_color * mesh.color.rgb;
    
    return vec4<f32>(1.0,0.0,1.0, 1.0);
}