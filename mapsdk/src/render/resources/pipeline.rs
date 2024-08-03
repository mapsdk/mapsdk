use std::borrow::Cow;

use wgpu::*;

use crate::render::{
    create_image_texture_bgl,
    resources::bind_group::{
        create_image_params_bgl, create_map_view_bgl, create_shape_fill_params_bgl,
        create_shape_stroke_params_bgl,
    },
    RenderingContext,
};

pub fn create_image_pipeline(rendering_context: &RenderingContext) -> RenderPipeline {
    let RenderingContext {
        surface,
        adapter,
        device,
        ..
    } = &rendering_context;

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Image Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../wgsl/image.wgsl"))),
    });

    let map_view_bgl = create_map_view_bgl(&rendering_context);
    let image_texture_bgl = create_image_texture_bgl(&rendering_context);
    let image_params_bgl = create_image_params_bgl(&rendering_context);

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Image PipelineLayout"),
        bind_group_layouts: &[&map_view_bgl, &image_texture_bgl, &image_params_bgl],
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
        label: Some("Image Pipeline"),
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
            front_face: FrontFace::Cw,
            cull_mode: Some(Face::Back),
            ..Default::default()
        },
        // depth_stencil: Some(DepthStencilState {
        //     format: TextureFormat::Depth32Float,
        //     depth_write_enabled: true,
        //     depth_compare: CompareFunction::Less,
        //     stencil: StencilState::default(),
        //     bias: DepthBiasState::default(),
        // }),
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            ..Default::default()
        },
        multiview: None,
        cache: None,
    })
}

pub fn create_shape_fill_pipeline(rendering_context: &RenderingContext) -> RenderPipeline {
    let RenderingContext {
        surface,
        adapter,
        device,
        ..
    } = &rendering_context;

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Shape Fill Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../wgsl/shape_fill.wgsl"))),
    });

    let map_view_bgl = create_map_view_bgl(&rendering_context);
    let shape_fill_params_bgl = create_shape_fill_params_bgl(&rendering_context);

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Shape Fill PipelineLayout"),
        bind_group_layouts: &[&map_view_bgl, &shape_fill_params_bgl],
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
        label: Some("Shape Fill Pipeline"),
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
            front_face: FrontFace::Cw,
            cull_mode: Some(Face::Back),
            ..Default::default()
        },
        // depth_stencil: Some(DepthStencilState {
        //     format: TextureFormat::Depth32Float,
        //     depth_write_enabled: true,
        //     depth_compare: CompareFunction::Less,
        //     stencil: StencilState::default(),
        //     bias: DepthBiasState::default(),
        // }),
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            ..Default::default()
        },
        multiview: None,
        cache: None,
    })
}

pub fn create_shape_stroke_pipeline(rendering_context: &RenderingContext) -> RenderPipeline {
    let RenderingContext {
        surface,
        adapter,
        device,
        ..
    } = &rendering_context;

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Shape Stroke Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../wgsl/shape_stroke.wgsl"))),
    });

    let map_view_bgl = create_map_view_bgl(&rendering_context);
    let shape_stroke_params_bgl = create_shape_stroke_params_bgl(&rendering_context);

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Shape Stroke PipelineLayout"),
        bind_group_layouts: &[&map_view_bgl, &shape_stroke_params_bgl],
        push_constant_ranges: &[],
    });

    let vertex_buffer_layout = VertexBufferLayout {
        array_stride: std::mem::size_of::<[f32; 5]>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32],
    };

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Shape Stroke Pipeline"),
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
            front_face: FrontFace::Cw,
            cull_mode: Some(Face::Back),
            ..Default::default()
        },
        // depth_stencil: Some(DepthStencilState {
        //     format: TextureFormat::Depth32Float,
        //     depth_write_enabled: true,
        //     depth_compare: CompareFunction::Less,
        //     stencil: StencilState::default(),
        //     bias: DepthBiasState::default(),
        // }),
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            ..Default::default()
        },
        multiview: None,
        cache: None,
    })
}
