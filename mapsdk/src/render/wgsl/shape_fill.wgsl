struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Map View BindGroup
@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(1) var<uniform> map_center: vec2<f32>;
@group(0) @binding(2) var<uniform> map_res: f32;

@vertex
fn vs_main(
    @location(0) vertex_coord: vec3<f32>
    ) -> VertexOutput {
    var x = (vertex_coord[0] - map_center[0]) / map_res;
    var y = (vertex_coord[1] - map_center[1]) / map_res;
    var z = vertex_coord[2] / map_res;
    var position = view_proj * vec4<f32>(x, y, z, 1.0);

    let color = vec4<f32>(1.0, 0.0, 0.0, 0.5);

    return VertexOutput(position, color);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.color;
}
