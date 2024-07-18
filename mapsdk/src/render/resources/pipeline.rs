use std::borrow::Cow;

use wgpu::*;

use crate::render::{
    create_image_params_bgl, create_image_texture_bgl, resources::layout::create_camera_bgl,
    RenderingContext,
};

pub(crate) fn create_image_pipeline(rendering_context: &RenderingContext) -> RenderPipeline {
    let RenderingContext {
        surface,
        adapter,
        device,
        ..
    } = &rendering_context;

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Image shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../wgsl/image.wgsl"))),
    });

    let camera_bind_group_layout = create_camera_bgl(&rendering_context);
    let texture_bind_group_layout = create_image_texture_bgl(&rendering_context);
    let params_bind_group_layout = create_image_params_bgl(&rendering_context);

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Image pipeline layout"),
        bind_group_layouts: &[
            &camera_bind_group_layout,
            &texture_bind_group_layout,
            &params_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let vertex_buffer_layout = VertexBufferLayout {
        array_stride: std::mem::size_of::<[f32; 2]>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
    };

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Image pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[vertex_buffer_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: PrimitiveState {
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            ..Default::default()
        },
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
