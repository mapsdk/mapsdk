struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) edge_norm: vec2<f32>,
};

// Map View BindGroup
@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(1) var<uniform> map_center: vec2<f32>;
@group(0) @binding(2) var<uniform> map_res: f32;

// Params BindGroup
@group(1) @binding(0) var<uniform> z: f32;
@group(1) @binding(1) var<uniform> stroke_half_width: f32;
@group(1) @binding(2) var<uniform> stroke_color: vec4<f32>;

@vertex
fn vs_main(
    @location(0) vertex_coord: vec2<f32>,
    @location(1) dir_offset: vec2<f32>,
    @location(2) norm_offset: vec2<f32>,
    @location(3) edge_norm: vec2<f32>,
    ) -> VertexOutput {
    var p = (vertex_coord - map_center) / map_res;
    p += dir_offset * stroke_half_width;
    p += norm_offset * stroke_half_width;
    
    let position = view_proj * vec4<f32>(p.xy, z / map_res, 1.0);

    return VertexOutput(position, stroke_color, edge_norm);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = smoothstep(0.0, 1.0, 2.0 * (1.0 - length(vertex.edge_norm)));
    
    return vec4(vertex.color.xyz, alpha * vertex.color.w);
}
