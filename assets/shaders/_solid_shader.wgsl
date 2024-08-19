#import bevy_pbr::forward_io::VertexOutput


struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

// struct VertexOutput {

// };

// @vertex
// fn vertex(vertex: Vertex) -> VertexOutput {
//     var out: VertexOutput;
//     let world_from_local = get_world_from_local(vertex.instance_index);
//     out.clip_position = mesh2d_position_local_to_clip(world_from_local, vec4<f32>(vertex.position, 1.0));
//     out.color = vertex.color;
//     out.barycentric = vertex.barycentric;
//     return out;
// }

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // return material.color;
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

