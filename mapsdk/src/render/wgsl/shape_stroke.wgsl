struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) norm: vec2<f32>,
};

// Map View BindGroup
@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(1) var<uniform> map_center: vec2<f32>;
@group(0) @binding(2) var<uniform> map_res: f32;

// Params BindGroup
@group(1) @binding(0) var<uniform> z: f32;
@group(1) @binding(1) var<uniform> stroke_width: f32;
@group(1) @binding(2) var<uniform> stroke_color: vec4<f32>;

@vertex
fn vs_main(
    @location(0) vertex_coord: vec2<f32>,
    @location(1) vertex_norm: vec2<f32>,
    @location(2) vertex_angle: f32
    ) -> VertexOutput {
    var x = (vertex_coord[0] - map_center[0]) / map_res;
    var y = (vertex_coord[1] - map_center[1]) / map_res;
    var hw = stroke_width / 2.0 / cos(vertex_angle / 2.0);
    var p = vec2<f32>(x, y) + vertex_norm * hw;

    var position = view_proj * vec4<f32>(p[0], p[1], z / map_res, 1.0);

    return VertexOutput(position, stroke_color, vertex_norm);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = smoothstep(0.0, 1.0, (1.0 - length(vertex.norm)) / 0.5);
    
    return vec4(vertex.color.xyz, alpha * vertex.color.w);
}
