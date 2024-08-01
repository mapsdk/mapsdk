use wgpu::*;

use crate::render::{
    resources::buffer::{
        create_uniform_buffer_from_f32_slice, create_uniform_buffer_from_vec4_f32_slice,
    },
    Camera, MapState, RenderingContext,
};

pub fn create_map_view_bg(
    rendering_context: &RenderingContext,
    layout: &BindGroupLayout,
    camera: &Camera,
    map_state: &MapState,
) -> BindGroup {
    let camera_buffer = create_uniform_buffer_from_vec4_f32_slice(
        rendering_context,
        "Camera buffer",
        &camera.view_proj(),
    );

    let map_center_buffer = create_uniform_buffer_from_f32_slice(
        rendering_context,
        "MapCenter Buffer",
        &[map_state.center.x as f32, map_state.center.y as f32],
    );

    let map_res_buffer = create_uniform_buffer_from_f32_slice(
        rendering_context,
        "MapRes Buffer",
        &[(map_state.zoom_res * map_state.map_res_ratio) as f32],
    );

    rendering_context
        .device
        .create_bind_group(&BindGroupDescriptor {
            label: Some("Map View BindGroup"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: map_center_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: map_res_buffer.as_entire_binding(),
                },
            ],
        })
}

pub fn create_map_view_bgl(rendering_context: &RenderingContext) -> BindGroupLayout {
    rendering_context
        .device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Map View BindGroupLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
}

pub fn create_image_texture_bg(
    rendering_context: &RenderingContext,
    layout: &BindGroupLayout,
    texture_view: &TextureView,
    sampler: &Sampler,
) -> BindGroup {
    rendering_context
        .device
        .create_bind_group(&BindGroupDescriptor {
            label: Some("Image Texture BindGroup"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
        })
}

pub fn create_image_texture_bgl(rendering_context: &RenderingContext) -> BindGroupLayout {
    rendering_context
        .device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Image Texture BindGroupLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
}
