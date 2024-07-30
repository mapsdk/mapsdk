use wgpu::*;

use crate::render::RenderingContext;

pub(crate) fn create_camera_bgl(rendering_context: &RenderingContext) -> BindGroupLayout {
    rendering_context
        .device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera BindGroupLayout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
}

pub(crate) fn create_image_texture_bgl(rendering_context: &RenderingContext) -> BindGroupLayout {
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

pub(crate) fn create_image_params_bgl(rendering_context: &RenderingContext) -> BindGroupLayout {
    rendering_context
        .device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Image Params BindGroupLayout"),
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
            ],
        })
}
