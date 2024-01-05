use bevy::ecs::system::Resource;
use bevy::ecs::world::FromWorld;
use bevy::prelude::World;

#[derive(Debug, Resource)]
pub struct Noise {
    // blue_noise: Texture,
    // blue_noise_sobol: StorageBuffer<Vec<i32>>,
    // blue_noise_scrambling_tile: StorageBuffer<Vec<i32>>,
    // blue_noise_ranking_tile: StorageBuffer<Vec<i32>>,
    // flushed: bool,
}

impl Noise {
    // pub fn bind_blue_noise(&self) -> impl Bindable + '_ {
    //     self.blue_noise.bind_readable()
    // }

    // pub fn bind_blue_noise_sobol(&self) -> impl Bindable + '_ {
    //     self.blue_noise_sobol.bind_readable()
    // }

    // pub fn bind_blue_noise_scrambling_tile(&self) -> impl Bindable + '_ {
    //     self.blue_noise_scrambling_tile.bind_readable()
    // }

    // pub fn bind_blue_noise_ranking_tile(&self) -> impl Bindable + '_ {
    //     self.blue_noise_ranking_tile.bind_readable()
    // }

    // pub fn flush(&mut self, device: &Device, queue: &Queue) {
    //     if self.flushed {
    //         return;
    //     }

    //     let bytes = include_bytes!("noise/blue-noise.png");

    //     let img = ImageReader::new(Cursor::new(bytes))
    //         .with_guessed_format()
    //         .unwrap()
    //         .decode()
    //         .unwrap();

    //     let img = img.as_rgba8().unwrap().as_raw();

    //     queue.write_texture(
    //         ImageCopyTexture {
    //             texture: self.blue_noise.tex(),
    //             mip_level: 0,
    //             origin: Origin3d { x: 0, y: 0, z: 0 },
    //             aspect: TextureAspect::All,
    //         },
    //         img,
    //         ImageDataLayout {
    //             offset: 0,
    //             bytes_per_row: Some(256 * 4),
    //             rows_per_image: None,
    //         },
    //         Extent3d {
    //             width: 256,
    //             height: 256,
    //             depth_or_array_layers: 1,
    //         },
    //     );

    //     // ---

    //     _ = self.blue_noise_sobol.flush(device, queue);
    //     _ = self.blue_noise_scrambling_tile.flush(device, queue);
    //     _ = self.blue_noise_ranking_tile.flush(device, queue);

    //     // ---

    //     self.flushed = true;
    // }
}

impl FromWorld for Noise {
    fn from_world(_: &mut World) -> Self {
        todo!()
    }
}
