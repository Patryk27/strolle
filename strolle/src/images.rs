use std::collections::HashMap;
use std::mem;

use derivative::Derivative;
use glam::{uvec2, vec4, Vec4};
use guillotiere::{size2, Allocation, AtlasAllocator};
use log::warn;
use wgpu::TextureFormat;

use crate::utils::ToGpu;
use crate::{Bindable, Image, ImageData, Params, Texture};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Images<P>
where
    P: Params,
{
    #[derivative(Debug = "ignore")]
    atlas: AtlasAllocator,
    atlas_texture: Texture,
    atlas_changes: Vec<AtlasChange<P>>,
    images: HashMap<P::ImageHandle, Allocation>,
    dynamic_textures: Vec<(P::ImageTexture, Allocation)>,
}

fn convert_to_rgba8unorm_srgb(
    data: &[u8],
    src_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let pixel_count = (width * height) as usize;
    let mut converted_data = Vec::with_capacity(pixel_count * 4);

    match src_format {
        wgpu::TextureFormat::R8Unorm => {
            for &r in data {
                converted_data.extend_from_slice(&[r, 0, 0, 255]);
            }
        }
        wgpu::TextureFormat::Rg8Unorm => {
            for chunk in data.chunks_exact(2) {
                let r = chunk[0];
                let g = chunk[1];
                converted_data.extend_from_slice(&[r, g, 0, 255]);
            }
        }
        wgpu::TextureFormat::Rgba8UnormSrgb => {
            converted_data.extend_from_slice(data);
        }
        _ => {
            println!("Unhandled texture format: {:?}", src_format);
        }
    }

    converted_data
}

impl<P> Images<P>
where
    P: Params,
{
    const ATLAS_WIDTH: u32 = 8192;
    const ATLAS_HEIGHT: u32 = 8192;

    pub fn new(device: &wgpu::Device) -> Self {
        let atlas = AtlasAllocator::new(size2(
            Self::ATLAS_WIDTH as i32,
            Self::ATLAS_HEIGHT as i32,
        ));

        let atlas_texture = Texture::builder("atlas")
            .with_size(uvec2(Self::ATLAS_WIDTH, Self::ATLAS_HEIGHT).to_gpu())
            .with_format(wgpu::TextureFormat::Rgba8UnormSrgb)
            .with_usage(wgpu::TextureUsages::TEXTURE_BINDING)
            .with_usage(wgpu::TextureUsages::COPY_DST)
            .build(device);

        Self {
            atlas,
            atlas_texture,
            atlas_changes: Default::default(),
            images: Default::default(),
            dynamic_textures: Default::default(),
        }
    }

    pub fn insert(&mut self, handle: P::ImageHandle, item: Image<P>) {
        let size = size2(
            item.texture_descriptor.size.width as i32,
            item.texture_descriptor.size.height as i32,
        );

        let alloc = if let Some(alloc) = self.images.get(&handle) {
            if size == alloc.rectangle.size() {
                Some(*alloc)
            } else {
                self.atlas.deallocate(alloc.id);
                self.atlas.allocate(size)
            }
        } else {
            self.atlas.allocate(size)
        };

        let Some(alloc) = alloc else {
            // TODO allocate new atlas, up to 16 (Metal's limit)
            warn!(
                "Cannot add image `{:?}` - no more space in the atlas",
                handle
            );

            return;
        };

        self.images.insert(handle, alloc);

        match item.data {
            data @ (ImageData::Raw { .. }
            | ImageData::Texture {
                is_dynamic: false, ..
            }) => {
                match &data {
                    ImageData::Raw { data } => {
                        // Convert data to atlas format (Rgba8UnormSrgb)
                        let converted_data = convert_to_rgba8unorm_srgb(
                            data,
                            item.texture_descriptor.format,
                            item.texture_descriptor.size.width,
                            item.texture_descriptor.size.height,
                        );

                        // Use atlas texture's format for calculations
                        let block_size = 4; // Rgba8UnormSrgb has 4 bytes per pixel
                        let unpadded_bytes_per_row =
                            item.texture_descriptor.size.width * block_size;
                        let padding =
                            (256 - (unpadded_bytes_per_row % 256)) % 256;
                        let padded_bytes_per_row =
                            unpadded_bytes_per_row + padding;

                        // Pad the converted data if necessary
                        let mut padded_data = Vec::with_capacity(
                            (padded_bytes_per_row
                                * item.texture_descriptor.size.height)
                                as usize,
                        );
                        if padding == 0 {
                            padded_data = converted_data;
                        } else {
                            let row_length = (unpadded_bytes_per_row) as usize;
                            for row in converted_data.chunks_exact(row_length) {
                                padded_data.extend_from_slice(row);
                                padded_data.extend(
                                    std::iter::repeat(0).take(padding as usize),
                                );
                            }
                        }

                        self.atlas_changes.push(AtlasChange::Set {
                            x: alloc.rectangle.min.x as u32,
                            y: alloc.rectangle.min.y as u32,
                            w: alloc.rectangle.width() as u32,
                            h: alloc.rectangle.height() as u32,
                            data: ImageData::Raw { data: padded_data },
                        });
                    }
                    ImageData::Texture { texture, .. } => {
                        self.atlas_changes.push(AtlasChange::Set {
                            x: alloc.rectangle.min.x as u32,
                            y: alloc.rectangle.min.y as u32,
                            w: alloc.rectangle.width() as u32,
                            h: alloc.rectangle.height() as u32,
                            data: data,
                        });
                    }
                };
            }

            ImageData::Texture {
                texture,
                is_dynamic: true,
            } => {
                self.dynamic_textures.push((texture, alloc));
            }
        }
    }

    pub fn remove(&mut self, handle: P::ImageHandle) {
        let Some(alloc) = self.images.remove(&handle) else {
            return;
        };

        self.atlas.deallocate(alloc.id);
    }

    pub fn lookup(&self, handle: P::ImageHandle) -> Option<Vec4> {
        self.images.get(&handle).map(|alloc| {
            vec4(
                alloc.rectangle.min.x as f32 / (Self::ATLAS_WIDTH as f32),
                alloc.rectangle.min.y as f32 / (Self::ATLAS_HEIGHT as f32),
                alloc.rectangle.width() as f32 / (Self::ATLAS_WIDTH as f32),
                alloc.rectangle.height() as f32 / (Self::ATLAS_HEIGHT as f32),
            )
        })
    }

    pub fn lookup_opt(&self, handle: Option<P::ImageHandle>) -> Option<Vec4> {
        self.lookup(handle?)
    }

    pub fn flush(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder = None;

        for change in mem::take(&mut self.atlas_changes) {
            match change {
                AtlasChange::Set { x, y, w, h, data } => {
                    let size = wgpu::Extent3d {
                        width: w,
                        height: h,
                        depth_or_array_layers: 1,
                    };

                    match data {
                        ImageData::Raw { data } => {
                            // Use atlas texture's format for calculations
                            let block_size = 4; // Rgba8UnormSrgb has 4 bytes per pixel
                            let unpadded_bytes_per_row = w * block_size;
                            let padding =
                                (256 - (unpadded_bytes_per_row % 256)) % 256;
                            let padded_bytes_per_row =
                                unpadded_bytes_per_row + padding;

                            println!(
                                "Writing texture: pos={}x{}, size={}x{}, data_len={}, bytes_per_row={}, padded_bytes_per_row={}",
                                x,
                                y,
                                w,
                                h,
                                data.len(),
                                unpadded_bytes_per_row,
                                padded_bytes_per_row
                            );

                            queue.write_texture(
                                wgpu::ImageCopyTexture {
                                    texture: self.atlas_texture.tex(),
                                    mip_level: 0,
                                    origin: wgpu::Origin3d { x, y, z: 0 },
                                    aspect: wgpu::TextureAspect::All,
                                },
                                &data,
                                wgpu::ImageDataLayout {
                                    offset: 0,
                                    bytes_per_row: Some(padded_bytes_per_row),
                                    rows_per_image: None,
                                },
                                size,
                            );
                        }

                        ImageData::Texture { texture, .. } => {
                            let encoder = encoder.get_or_insert_with(|| {
                                device.create_command_encoder(
                                    &wgpu::CommandEncoderDescriptor {
                                        label: Some("strolle_atlas"),
                                    },
                                )
                            });

                            encoder.copy_texture_to_texture(
                                texture.as_image_copy(),
                                wgpu::ImageCopyTexture {
                                    texture: self.atlas_texture.tex(),
                                    mip_level: 0,
                                    origin: wgpu::Origin3d { x, y, z: 0 },
                                    aspect: wgpu::TextureAspect::All,
                                },
                                size,
                            );
                        }
                    }
                }
            }
        }

        for (tex, alloc) in &self.dynamic_textures {
            let encoder = encoder.get_or_insert_with(|| {
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("strolle_atlas"),
                })
            });

            encoder.copy_texture_to_texture(
                tex.as_image_copy(),
                wgpu::ImageCopyTexture {
                    texture: self.atlas_texture.tex(),
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: alloc.rectangle.min.x as u32,
                        y: alloc.rectangle.min.y as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: alloc.rectangle.size().width as u32,
                    height: alloc.rectangle.size().height as u32,
                    depth_or_array_layers: 1,
                },
            )
        }

        if let Some(encoder) = encoder {
            queue.submit([encoder.finish()]);
        }
    }

    pub fn bind_atlas(&self) -> impl Bindable + '_ {
        self.atlas_texture.bind_sampled()
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
enum AtlasChange<P>
where
    P: Params,
{
    Set {
        x: u32,
        y: u32,
        w: u32,
        h: u32,

        #[derivative(Debug = "ignore")]
        data: ImageData<P>,
    },
}
