mod buffers;
mod passes;

use std::ops::DerefMut;

use log::{debug, info};
use rand::Rng;

pub use self::buffers::*;
pub use self::passes::*;
use crate::{
    Camera, CameraMode, Engine, Event, EventHandler, EventHandlerContext,
    Params,
};

#[derive(Debug)]
pub struct CameraController<P>
where
    P: Params,
{
    camera: Camera,
    buffers: CameraBuffers,
    passes: CameraPasses<P>,
    frame: u32,
}

impl<P> CameraController<P>
where
    P: Params,
{
    pub(crate) fn new(
        engine: &Engine<P>,
        device: &wgpu::Device,
        camera: Camera,
    ) -> Self {
        info!("Creating camera: {}", camera.describe());

        let buffers = CameraBuffers::new(device, &camera);
        let passes = CameraPasses::new(engine, device, &camera, &buffers);

        debug!("Camera created");

        Self {
            camera,
            buffers,
            passes,
            frame: 0,
        }
    }

    pub fn update(
        &mut self,
        engine: &Engine<P>,
        device: &wgpu::Device,
        camera: Camera,
    ) {
        let needs_rebuilding = self.camera.is_invalidated_by(&camera);

        self.camera = camera;
        *self.buffers.camera.deref_mut() = self.camera.serialize();

        if needs_rebuilding {
            self.rebuild_buffers(device);
            self.rebuild_passes(engine, device);
        }
    }

    fn rebuild_buffers(&mut self, device: &wgpu::Device) {
        debug!("Rebuilding buffers for camera: {}", self.camera.describe());

        self.buffers = CameraBuffers::new(device, &self.camera);
    }

    fn rebuild_passes(&mut self, engine: &Engine<P>, device: &wgpu::Device) {
        debug!("Rebuilding passes for camera: {}", self.camera.describe());

        self.passes =
            CameraPasses::new(engine, device, &self.camera, &self.buffers);
    }

    pub fn flush(&mut self, queue: &wgpu::Queue) {
        self.frame += 1;
        self.buffers.camera.flush(queue);
    }

    pub fn render(
        &self,
        engine: &Engine<P>,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        if let CameraMode::DisplayBvhHeatmap = self.camera.mode {
            self.passes.ray_tracing.run(self, encoder);
            self.passes.drawing.run(self, encoder, view);
        } else {
            self.passes.raster.run(engine, self, encoder);

            if self.camera.mode.needs_indirect_lightning() {
                let seed = rand::thread_rng().gen();

                self.passes.voxel_tracing.run(self, encoder, seed);
                self.passes.voxel_shading.run(self, encoder, seed);
                self.passes.voxel_painting.run(self, encoder);
            }

            self.passes.ray_shading.run(self, encoder);
            self.passes.denoising.run(self, encoder);
            self.passes.drawing.run(self, encoder, view);
        }
    }
}

impl<P> EventHandler<P> for CameraController<P>
where
    P: Params,
{
    fn handle(&mut self, ctxt: EventHandlerContext<P>) {
        if let Event::ImageChanged(_) | Event::ImageRemoved(_) = ctxt.event {
            self.rebuild_passes(ctxt.engine, ctxt.device);
        }

        self.passes.handle(ctxt);
    }
}

impl<P> Drop for CameraController<P>
where
    P: Params,
{
    fn drop(&mut self) {
        info!("Deleting camera: {}", self.camera.describe());
    }
}
