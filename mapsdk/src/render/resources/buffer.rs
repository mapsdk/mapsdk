use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};

use crate::render::{
    tessellation::{FillVertexIndex, StrokeVertexIndex},
    RenderingContext,
};

pub struct VertexIndexBuffer {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
}

impl VertexIndexBuffer {
    pub fn from_fill_vertex_index(
        rendering_context: &RenderingContext,
        fill_vertex_index: &FillVertexIndex,
    ) -> Self {
        let vertex_buffer = create_vertex_buffer_from_vec2_f32_slice(
            rendering_context,
            "Fill VertexBuffer",
            &fill_vertex_index.vertices,
        );

        let index_buffer = create_index_buffer_from_u16_slice(
            rendering_context,
            "Fill IndexBuffer",
            &fill_vertex_index.indices,
        );

        Self {
            vertex_buffer,
            index_buffer,
            index_count: fill_vertex_index.indices.len() as u32,
        }
    }
    pub fn from_stroke_vertex_index(
        rendering_context: &RenderingContext,
        stroke_vertex_index: &StrokeVertexIndex,
    ) -> Self {
        let vertex_buffer = create_vertex_buffer_from_vec8_f32_slice(
            rendering_context,
            "Stroke VertexBuffer",
            &stroke_vertex_index.vertices,
        );

        let index_buffer = create_index_buffer_from_u16_slice(
            rendering_context,
            "Stroke IndexBuffer",
            &stroke_vertex_index.indices,
        );

        Self {
            vertex_buffer,
            index_buffer,
            index_count: stroke_vertex_index.indices.len() as u32,
        }
    }
}

pub fn create_index_buffer_from_u16_slice(
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

pub fn create_uniform_buffer_from_f32_slice(
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

pub fn create_uniform_buffer_from_vec4_f32_slice(
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

pub fn create_vertex_buffer_from_vec2_f32_slice(
    rendering_context: &RenderingContext,
    label: &str,
    slice: &[[f32; 2]],
) -> Buffer {
    rendering_context
        .device
        .create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&slice),
            usage: BufferUsages::VERTEX,
        })
}

pub fn create_vertex_buffer_from_vec4_f32_slice(
    rendering_context: &RenderingContext,
    label: &str,
    slice: &[[f32; 4]],
) -> Buffer {
    rendering_context
        .device
        .create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&slice),
            usage: BufferUsages::VERTEX,
        })
}

pub fn create_vertex_buffer_from_vec8_f32_slice(
    rendering_context: &RenderingContext,
    label: &str,
    slice: &[[f32; 8]],
) -> Buffer {
    rendering_context
        .device
        .create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&slice),
            usage: BufferUsages::VERTEX,
        })
}
