struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coord: vec2<f32>,
};

// Map View BindGroup
@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(1) var<uniform> map_center: vec2<f32>;
@group(0) @binding(2) var<uniform> map_res: f32;

// Texture BindGroup
@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

// Image Params
@group(2) @binding(0) var<uniform> z: f32;

@vertex
fn vs_main(
    @location(0) vertex_coord: vec2<f32>,
    @builtin(vertex_index) vertex_idx: u32
    ) -> VertexOutput {
    var p = (vertex_coord - map_center) / map_res;
    let position = view_proj * vec4<f32>(p.xy, z / map_res, 1.0);

    var texture_coords = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0)
    );
    let texture_coord = texture_coords[vertex_idx];

    return VertexOutput(position, texture_coord);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, vertex.texture_coord);
}
