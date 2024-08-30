use image::DynamicImage;
use wgpu::*;

pub fn create_depth_texture(device: &Device, width: u32, height: u32) -> Texture {
    let texture_size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture_desc = TextureDescriptor {
        label: Some("Depth Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };

    device.create_texture(&texture_desc)
}

pub fn create_texture(device: &Device, width: u32, height: u32, format: TextureFormat) -> Texture {
    let texture_size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture_desc = TextureDescriptor {
        label: Some("Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::COPY_SRC
            | TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };

    device.create_texture(&texture_desc)
}
