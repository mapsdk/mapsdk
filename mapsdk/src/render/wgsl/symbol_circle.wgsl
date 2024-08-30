struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) coord: vec2<f32>,
    @location(1) color: vec4<f32>,
};

// Map View BindGroup
@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(1) var<uniform> map_center: vec2<f32>;
@group(0) @binding(2) var<uniform> map_res: f32;

// Params BindGroup
@group(1) @binding(0) var<uniform> z: f32;
@group(1) @binding(1) var<uniform> radius: f32;
@group(1) @binding(2) var<uniform> fill_color: vec4<f32>;

@vertex
fn vs_main(
    @location(0) vertex_coord: vec2<f32>,
    @builtin(vertex_index) vertex_idx: u32
    ) -> VertexOutput {
    var x = (vertex_coord[0] - map_center[0]) / map_res;
    var y = (vertex_coord[1] - map_center[1]) / map_res;

    let vertex_idx_norm = vertex_idx % 4;
    let dx = select(radius * 2.0, 0.0, vertex_idx_norm % 2 == 0) - radius;
    let dy = radius - select(radius * 2.0, 0.0, vertex_idx_norm < 2);

    let position = view_proj * vec4<f32>(x + dx, y + dy, z / map_res, 1.0);
    let coord = vec2<f32>(dx, dy);

    return VertexOutput(position, coord, fill_color);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    if (length(vertex.coord) > radius) {
        discard;
    }

    return vertex.color;
}
