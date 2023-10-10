use std::io::Cursor;

use image::io::Reader as ImageReader;

use crate::{gpu, Bindable, Texture};

#[derive(Debug)]
pub struct Noise {
    blue_noise: Texture,
    flushed: bool,
}

impl Noise {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            blue_noise: Texture::builder("blue_noise")
                .with_size(gpu::BlueNoise::SIZE)
                .with_format(wgpu::TextureFormat::Rgba8Unorm)
                .with_usage(wgpu::TextureUsages::COPY_DST)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .build(device),
            flushed: false,
        }
    }

    pub fn bind_blue_noise_texture(&self) -> impl Bindable + '_ {
        self.blue_noise.bind_readable()
    }

    pub fn flush(&mut self, queue: &wgpu::Queue) {
        if self.flushed {
            return;
        }

        let bytes = include_bytes!("../assets/blue-noise.png");

        let img = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        let img = img.as_rgba8().unwrap().as_raw();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: self.blue_noise.tex(),
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            img,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(256 * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
        );

        self.flushed = true;
    }
}
