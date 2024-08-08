use wgpu::*;

use crate::{
    feature::style::ShapeStyles,
    render::{
        resources::buffer::{
            create_uniform_buffer_from_f32_slice, create_uniform_buffer_from_vec4_f32_slice,
        },
        Camera, MapState, RenderingContext,
    },
};

pub fn create_image_params_bg(
    rendering_context: &RenderingContext,
    layout: &BindGroupLayout,
    z: f32,
) -> BindGroup {
    let z_buffer = create_uniform_buffer_from_f32_slice(rendering_context, "Z Buffer", &[z]);

    rendering_context
        .device
        .create_bind_group(&BindGroupDescriptor {
            label: Some("Image Params BindGroup"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: z_buffer.as_entire_binding(),
            }],
        })
}

pub fn create_image_params_bgl(rendering_context: &RenderingContext) -> BindGroupLayout {
    rendering_context
        .device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Image Params BindGroupLayout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
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

pub fn create_map_view_bg(
    rendering_context: &RenderingContext,
    layout: &BindGroupLayout,
    camera: &Camera,
    map_state: &MapState,
) -> BindGroup {
    let camera_buffer = create_uniform_buffer_from_vec4_f32_slice(
        rendering_context,
        "Camera Buffer",
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
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
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

pub fn create_shape_fill_params_bg(
    rendering_context: &RenderingContext,
    layout: &BindGroupLayout,
    z: f32,
    shape_styles: &ShapeStyles,
) -> BindGroup {
    let z_buffer = create_uniform_buffer_from_f32_slice(rendering_context, "Z Buffer", &[z]);

    let fill_color: [f32; 4] = shape_styles.fill_color.clone().into();
    let fill_color_buffer =
        create_uniform_buffer_from_f32_slice(rendering_context, "Fill Color Buffer", &fill_color);

    rendering_context
        .device
        .create_bind_group(&BindGroupDescriptor {
            label: Some("Shape Fill Params BindGroup"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: z_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: fill_color_buffer.as_entire_binding(),
                },
            ],
        })
}

pub fn create_shape_fill_params_bgl(rendering_context: &RenderingContext) -> BindGroupLayout {
    rendering_context
        .device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Shape Fill Params BindGroupLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
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

pub fn create_shape_stroke_params_bg(
    rendering_context: &RenderingContext,
    layout: &BindGroupLayout,
    z: f32,
    shape_styles: &ShapeStyles,
) -> BindGroup {
    let pixel_ratio = rendering_context.pixel_ratio;

    let z_buffer = create_uniform_buffer_from_f32_slice(rendering_context, "Z Buffer", &[z]);

    let stroke_half_width_buffer = create_uniform_buffer_from_f32_slice(
        rendering_context,
        "Stroke Half Width Buffer",
        &[0.5 * shape_styles.stroke_width * pixel_ratio as f32],
    );

    let stroke_color: [f32; 4] = shape_styles.stroke_color.clone().into();
    let stroke_color_buffer = create_uniform_buffer_from_f32_slice(
        rendering_context,
        "Stroke Color Buffer",
        &stroke_color,
    );

    rendering_context
        .device
        .create_bind_group(&BindGroupDescriptor {
            label: Some("Shape Stroke Params BindGroup"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: z_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: stroke_half_width_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: stroke_color_buffer.as_entire_binding(),
                },
            ],
        })
}

pub fn create_shape_stroke_params_bgl(rendering_context: &RenderingContext) -> BindGroupLayout {
    rendering_context
        .device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Shape Stroke Params BindGroupLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
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

pub fn create_symbol_circle_params_bg(
    rendering_context: &RenderingContext,
    layout: &BindGroupLayout,
    z: f32,
    shape_styles: &ShapeStyles,
) -> BindGroup {
    let pixel_ratio = rendering_context.pixel_ratio;

    let z_buffer = create_uniform_buffer_from_f32_slice(rendering_context, "Z Buffer", &[z]);

    let radius = shape_styles.symbol_size * pixel_ratio as f32 / 2.0;
    let radius_buffer =
        create_uniform_buffer_from_f32_slice(rendering_context, "Radius Buffer", &[radius]);

    let fill_color: [f32; 4] = shape_styles.fill_color.clone().into();
    let fill_color_buffer =
        create_uniform_buffer_from_f32_slice(rendering_context, "Fill Color Buffer", &fill_color);

    rendering_context
        .device
        .create_bind_group(&BindGroupDescriptor {
            label: Some("Symbol Rect Params BindGroup"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: z_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: radius_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: fill_color_buffer.as_entire_binding(),
                },
            ],
        })
}

pub fn create_symbol_circle_params_bgl(rendering_context: &RenderingContext) -> BindGroupLayout {
    rendering_context
        .device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Symbol Rect Params BindGroupLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
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
