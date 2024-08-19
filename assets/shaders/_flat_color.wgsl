// solid color, from uniform variable

#import bevy_pbr::mesh_view_bindings

struct ColorMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: ColorMaterial;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    return material.color;
}