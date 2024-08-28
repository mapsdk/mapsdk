use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};

use crate::render::tessellation::{FillVertexIndex, StrokeVertexIndex};

pub struct VertexIndexBuffer {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
}

impl VertexIndexBuffer {
    pub fn from_fill_vertex_index(device: &Device, fill_vertex_index: &FillVertexIndex) -> Self {
        let vertex_buffer = create_vertex_buffer_from_vec2_f32_slice(
            device,
            "Fill VertexBuffer",
            &fill_vertex_index.vertices,
        );

        let index_buffer = create_index_buffer_from_u16_slice(
            device,
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
        device: &Device,
        stroke_vertex_index: &StrokeVertexIndex,
    ) -> Self {
        let vertex_buffer = create_vertex_buffer_from_vec7_f32_slice(
            device,
            "Stroke VertexBuffer",
            &stroke_vertex_index.vertices,
        );

        let index_buffer = create_index_buffer_from_u16_slice(
            device,
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

pub fn create_index_buffer_from_u16_slice(device: &Device, label: &str, slice: &[u16]) -> Buffer {
    device.create_buffer_init(&BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&slice),
        usage: BufferUsages::INDEX,
    })
}

pub fn create_uniform_buffer_from_f32_slice(device: &Device, label: &str, slice: &[f32]) -> Buffer {
    device.create_buffer_init(&BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&slice),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

pub fn create_uniform_buffer_from_u32_slice(device: &Device, label: &str, slice: &[u32]) -> Buffer {
    device.create_buffer_init(&BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&slice),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

pub fn create_uniform_buffer_from_vec4_f32_slice(
    device: &Device,
    label: &str,
    slice: &[[f32; 4]],
) -> Buffer {
    device.create_buffer_init(&BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&slice),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

pub fn create_vertex_buffer_from_vec2_f32_slice(
    device: &Device,
    label: &str,
    slice: &[[f32; 2]],
) -> Buffer {
    device.create_buffer_init(&BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&slice),
        usage: BufferUsages::VERTEX,
    })
}

pub fn create_vertex_buffer_from_vec7_f32_slice(
    device: &Device,
    label: &str,
    slice: &[[f32; 7]],
) -> Buffer {
    device.create_buffer_init(&BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&slice),
        usage: BufferUsages::VERTEX,
    })
}
