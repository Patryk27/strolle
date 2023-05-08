mod buffers;
mod passes;

use std::ops::DerefMut;

use log::{debug, info};
use rand::Rng;

pub use self::buffers::*;
pub use self::passes::*;
use crate::{Camera, CameraMode, Engine, Params};

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
        let is_invalidated = self.camera.is_invalidated_by(&camera);

        self.camera = camera;
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

    pub fn on_buffers_reallocated(
        &mut self,
        engine: &Engine<P>,
        device: &wgpu::Device,
    ) {
        // TODO knowing which passes to re-allocate in here requires knowing
        //      which buffers those passes use in their `::new()` so currently
        //      it's pretty error-prone; it would be nice if the passes could
        //      react to the changes autonomously

        self.passes.ray_shading =
            RayShadingPass::new(engine, device, &self.buffers);

        self.passes.ray_tracing =
            RayTracingPass::new(engine, device, &self.buffers);

        self.passes.voxel_shading =
            VoxelShadingPass::new(engine, device, &self.buffers);

        self.passes.voxel_tracing =
            VoxelTracingPass::new(engine, device, &self.buffers);
    }

    pub fn on_image_changed(
        &mut self,
        engine: &Engine<P>,
        device: &wgpu::Device,
        image_handle: &P::ImageHandle,
    ) {
        self.passes
            .raster
            .on_image_changed(engine, device, image_handle);
    }

    pub fn on_image_removed(
        &mut self,
        engine: &Engine<P>,
        device: &wgpu::Device,
        image_handle: &P::ImageHandle,
    ) {
        self.passes
            .raster
            .on_image_removed(engine, device, image_handle);
    }

    pub fn on_images_modified(
        &mut self,
        engine: &Engine<P>,
        device: &wgpu::Device,
    ) {
        self.passes.ray_shading =
            RayShadingPass::new(engine, device, &self.buffers);

        self.passes.voxel_shading =
            VoxelShadingPass::new(engine, device, &self.buffers);
    }

    pub fn on_material_changed(
        &mut self,
        engine: &Engine<P>,
        device: &wgpu::Device,
        material_handle: &P::MaterialHandle,
    ) {
        self.passes
            .raster
            .on_material_changed(engine, device, material_handle);
    }

    pub fn on_material_removed(&mut self, material_handle: &P::MaterialHandle) {
        self.passes.raster.on_material_removed(material_handle);
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
