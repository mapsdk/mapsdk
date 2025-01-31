use std::borrow::Cow;

use wgpu::*;

use crate::render::{
    create_image_texture_bgl,
    resources::bind_group::{
        create_image_params_bgl, create_map_view_bgl, create_shape_fill_params_bgl,
        create_shape_stroke_params_bgl, create_symbol_circle_params_bgl,
    },
};

pub fn create_image_pipeline(
    device: &Device,
    color_target_state: &ColorTargetState,
) -> RenderPipeline {
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Image Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../wgsl/image.wgsl"))),
    });

    let map_view_bgl = create_map_view_bgl(device);
    let image_texture_bgl = create_image_texture_bgl(device);
    let image_params_bgl = create_image_params_bgl(device);

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
            targets: &[Some(color_target_state.clone())],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleStrip,
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

pub fn create_shape_fill_pipeline(
    device: &Device,
    color_target_state: &ColorTargetState,
) -> RenderPipeline {
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Shape Fill Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../wgsl/shape_fill.wgsl"))),
    });

    let map_view_bgl = create_map_view_bgl(device);
    let shape_fill_params_bgl = create_shape_fill_params_bgl(device);

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
            targets: &[Some(color_target_state.clone())],
        }),
        primitive: PrimitiveState::default(),
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

pub fn create_shape_stroke_pipeline(
    device: &Device,
    color_target_state: &ColorTargetState,
) -> RenderPipeline {
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Shape Stroke Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../wgsl/shape_stroke.wgsl"))),
    });

    let map_view_bgl = create_map_view_bgl(device);
    let shape_stroke_params_bgl = create_shape_stroke_params_bgl(device);

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Shape Stroke PipelineLayout"),
        bind_group_layouts: &[&map_view_bgl, &shape_stroke_params_bgl],
        push_constant_ranges: &[],
    });

    let vertex_buffer_layout = VertexBufferLayout {
        array_stride: std::mem::size_of::<[f32; 7]>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x2, 3 => Float32],
    };

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
            targets: &[Some(color_target_state.clone())],
        }),
        primitive: PrimitiveState::default(),
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

pub fn create_symbol_circle_pipeline(
    device: &Device,
    color_target_state: &ColorTargetState,
) -> RenderPipeline {
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Symbol Circle Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../wgsl/symbol_circle.wgsl"))),
    });

    let map_view_bgl = create_map_view_bgl(device);
    let symbol_circle_params_bgl = create_symbol_circle_params_bgl(device);

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Symbol Circle PipelineLayout"),
        bind_group_layouts: &[&map_view_bgl, &symbol_circle_params_bgl],
        push_constant_ranges: &[],
    });

    let vertex_buffer_layout = VertexBufferLayout {
        array_stride: std::mem::size_of::<[f32; 2]>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
    };

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Symbol Circle Pipeline"),
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
            targets: &[Some(color_target_state.clone())],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleStrip,
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
