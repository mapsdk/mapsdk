struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Map View BindGroup
@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(1) var<uniform> map_center: vec2<f32>;
@group(0) @binding(2) var<uniform> map_res: f32;

// Params BindGroup
@group(1) @binding(0) var<uniform> z: f32;
@group(1) @binding(1) var<uniform> fill_color: vec4<f32>;

@vertex
fn vs_main(
    @location(0) vertex_coord: vec2<f32>
    ) -> VertexOutput {
    var p = (vertex_coord - map_center) / map_res;
    let position = view_proj * vec4<f32>(p.xy, z / map_res, 1.0);

    return VertexOutput(position, fill_color);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.color;
}
