mod buffers;
mod pass;
mod passes;

use std::ops::DerefMut;

use log::{debug, info};
use rand::Rng;

pub use self::buffers::*;
pub use self::pass::*;
pub use self::passes::*;
use crate::{Camera, CameraMode, Engine, Params};

#[derive(Debug)]
pub struct CameraController {
    camera: Camera,
    buffers: CameraBuffers,
    passes: CameraPasses,
    frame: u32,
}

impl CameraController {
    pub(crate) fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        camera: Camera,
    ) -> Self
    where
        P: Params,
    {
        info!("Creating camera: {}", camera.describe());

        let buffers = CameraBuffers::new(device, &camera);
        let passes = CameraPasses::new(engine, device, &camera, &buffers);

        Self {
            camera,
            buffers,
            passes,
            frame: 0,
        }
    }

    pub fn update<P>(
        &mut self,
        engine: &Engine<P>,
        device: &wgpu::Device,
        camera: Camera,
    ) where
        P: Params,
    {
        let is_invalidated = self.camera.is_invalidated_by(&camera);

        self.camera = camera;
        *self.buffers.prev_camera.deref_mut() = *self.buffers.camera;
        *self.buffers.camera.deref_mut() = self.camera.serialize();

        if is_invalidated {
            self.rebuild_buffers(device);
            self.rebuild_passes(engine, device);
        }
    }

    fn rebuild_buffers(&mut self, device: &wgpu::Device) {
        debug!("Rebuilding buffers for camera: {}", self.camera.describe());

        self.buffers = CameraBuffers::new(device, &self.camera);
    }

    fn rebuild_passes<P>(&mut self, engine: &Engine<P>, device: &wgpu::Device)
    where
        P: Params,
    {
        debug!("Rebuilding passes for camera: {}", self.camera.describe());

        self.passes =
            CameraPasses::new(engine, device, &self.camera, &self.buffers);
    }

    pub fn flush(&mut self, queue: &wgpu::Queue) {
        self.frame += 1;
        self.buffers.camera.flush(queue);
        self.buffers.prev_camera.flush(queue);
    }

    pub fn render<P>(
        &self,
        engine: &Engine<P>,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) where
        P: Params,
    {
        let has_any_objects = !engine.instances.is_empty();

        if let CameraMode::BvhHeatmap = self.camera.mode {
            self.passes.bvh_heatmap.run(self, encoder);
            self.passes.output_drawing.run(self, encoder, view);
        } else {
            self.passes.atmosphere.run(engine, self, encoder);
            self.passes.direct_raster.run(engine, self, encoder);

            if has_any_objects {
                self.passes.reprojection.run(self, encoder);

                if self.camera.mode.needs_direct_lightning() {
                    self.passes.direct_secondary_tracing.run(self, encoder);
                    self.passes.direct_initial_shading.run(self, encoder);
                    self.passes.direct_temporal_resampling.run(self, encoder);
                    self.passes.direct_spatial_resampling.run(self, encoder);
                    self.passes.direct_resolving.run(self, encoder);
                    self.passes.direct_denoising.run(self, encoder);
                }

                if self.camera.mode.needs_indirect_lightning() {
                    let seed = rand::thread_rng().gen();

                    self.passes
                        .indirect_initial_tracing
                        .run(self, encoder, seed);

                    self.passes
                        .indirect_initial_shading
                        .run(self, encoder, seed);

                    self.passes.indirect_temporal_resampling.run(self, encoder);
                    self.passes.indirect_spatial_resampling.run(self, encoder);
                    self.passes.indirect_resolving.run(self, encoder);
                    self.passes.indirect_denoising.run(self, encoder);
                }
            }

            self.passes.output_drawing.run(self, encoder, view);
        }
    }

    pub fn invalidate<P>(&mut self, engine: &Engine<P>, device: &wgpu::Device)
    where
        P: Params,
    {
        self.rebuild_passes(engine, device);
    }

    /// Returns whether the current frame should use the first or the second
    /// resource, when given resource is double-buffered.
    fn is_alternate(&self) -> bool {
        self.frame % 2 == 1
    }
}

impl Drop for CameraController {
    fn drop(&mut self) {
        info!("Deleting camera: {}", self.camera.describe());
    }
}
