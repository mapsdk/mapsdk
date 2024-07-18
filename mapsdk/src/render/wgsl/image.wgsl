struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coord: vec2<f32>,
};

// Camera bind group
@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;

// Texture bind group
@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

// Params bind group
@group(2) @binding(0) var<uniform> map_center: vec2<f32>;
@group(2) @binding(1) var<uniform> map_res:  f32;

@vertex
fn vs_main(
    @location(0) vertex_coord: vec2<f32>,
    @builtin(vertex_index) vertex_idx: u32
    ) -> VertexOutput {
    var x = (vertex_coord[0] - map_center[0]) / map_res;
    var y = (vertex_coord[1] - map_center[1]) / map_res;
    var position = view_proj * vec4<f32>(x, y, 0.0, 1.0);

    var texture_coords = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );
    var texture_coord = texture_coords[vertex_idx];

    return VertexOutput(position, texture_coord);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, vertex.texture_coord);
}
