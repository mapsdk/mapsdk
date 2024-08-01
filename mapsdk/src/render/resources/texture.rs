use image::DynamicImage;
use wgpu::*;

use crate::render::RenderingContext;

pub fn create_depth_texture(
    rendering_context: &RenderingContext,
    width: u32,
    height: u32,
) -> Texture {
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

    rendering_context.device.create_texture(&texture_desc)
}

pub fn create_texture_from_image(
    rendering_context: &RenderingContext,
    image: &DynamicImage,
) -> Texture {
    let texture_size = Extent3d {
        width: image.width(),
        height: image.height(),
        depth_or_array_layers: 1,
    };

    let texture_desc = TextureDescriptor {
        label: Some("Image Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    };

    rendering_context.device.create_texture(&texture_desc)
}
