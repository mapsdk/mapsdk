use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};

use crate::render::RenderingContext;

pub(crate) fn create_index_buffer_from_u16_slice(
    rendering_context: &RenderingContext,
    label: &str,
    slice: &[u16],
) -> Buffer {
    rendering_context
        .device
        .create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&slice),
            usage: BufferUsages::INDEX,
        })
}

pub(crate) fn create_uniform_buffer_from_f32_slice(
    rendering_context: &RenderingContext,
    label: &str,
    slice: &[f32],
) -> Buffer {
    rendering_context
        .device
        .create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&slice),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        })
}

pub(crate) fn create_uniform_buffer_from_vec4_f32_slice(
    rendering_context: &RenderingContext,
    label: &str,
    slice: &[[f32; 4]],
) -> Buffer {
    rendering_context
        .device
        .create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&slice),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        })
}

pub(crate) fn create_vertex_buffer_from_vec3_f32_slice(
    rendering_context: &RenderingContext,
    label: &str,
    slice: &[[f32; 3]],
) -> Buffer {
    rendering_context
        .device
        .create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&slice),
            usage: BufferUsages::VERTEX,
        })
}
