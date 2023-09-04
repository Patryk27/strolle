mod buffers;
mod pass;
mod passes;

use std::ops::DerefMut;

use log::{debug, info};
use rand::Rng;

pub use self::buffers::*;
pub use self::pass::*;
pub use self::passes::*;
use crate::{gpu, Camera, CameraMode, Engine, Params};

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
        match self.camera.mode {
            CameraMode::BvhHeatmap => {
                self.passes.bvh_heatmap.run(self, encoder);
                self.passes.frame_composition.run(self, encoder, view);
            }

            CameraMode::Reference { depth } => {
                self.passes.atmosphere.run(engine, self, encoder);

                for depth in 0..=depth {
                    self.passes.reference_tracing.run(self, encoder, depth);
                    self.passes.reference_shading.run(self, encoder, depth);
                }

                self.passes.reference_shading.run(self, encoder, u8::MAX);
                self.passes.frame_composition.run(self, encoder, view);
            }

            _ => {
                let has_any_objects = !engine.instances.is_empty();

                self.passes.atmosphere.run(engine, self, encoder);
                self.passes.direct_raster.run(engine, self, encoder);

                if has_any_objects {
                    self.passes.frame_reprojection.run(self, encoder);

                    if self.camera.mode.needs_direct_lightning() {
                        if self.frame % 3 == 2 {
                            self.passes.direct_validation.run(self, encoder);
                        } else {
                            self.passes.direct_shading.run(self, encoder);

                            self.passes
                                .direct_temporal_resampling
                                .run(self, encoder);
                        }

                        self.passes
                            .direct_spatial_resampling
                            .run(self, encoder);

                        self.passes.direct_resolving.run(self, encoder);
                        self.passes.direct_denoising.run(self, encoder);
                    }

                    if self.camera.mode.needs_indirect_lightning() {
                        self.passes.indirect_tracing.run(self, encoder);
                        self.passes.indirect_shading.run(self, encoder);

                        self.passes
                            .indirect_diffuse_temporal_resampling
                            .run(self, encoder);

                        self.passes
                            .indirect_diffuse_spatial_resampling
                            .run(self, encoder);

                        self.passes
                            .indirect_diffuse_resolving
                            .run(self, encoder);

                        self.passes
                            .indirect_diffuse_denoising
                            .run(self, encoder);

                        self.passes
                            .indirect_specular_resampling
                            .run(self, encoder);

                        self.passes
                            .indirect_specular_resolving
                            .run(self, encoder);

                        self.passes
                            .indirect_specular_denoising
                            .run(self, encoder);
                    }
                }

                self.passes.frame_composition.run(self, encoder, view);
            }
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

    fn pass_params(&self) -> gpu::PassParams {
        gpu::PassParams {
            seed: rand::thread_rng().gen(),
            frame: self.frame,
        }
    }
}

impl Drop for CameraController {
    fn drop(&mut self) {
        info!("Deleting camera: {}", self.camera.describe());
    }
}
