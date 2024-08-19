#import bevy_pbr::mesh_view_bindingsssss
#import bevy_pbr::mesh_vertex_output

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    // Convert normal from [-1, 1] range to [0, 1] range
    // let color = (world_normal + 1.0) * 0.5;
    let color = vec3<f32>(0.0,1.0,0.0);
    return vec4<f32>(color, 1.0);
}