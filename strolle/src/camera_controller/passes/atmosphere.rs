use std::sync::Mutex;

use strolle_gpu as gpu;

use crate::{
    CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct AtmospherePass {
    generate_transmittance_lut_pass: CameraComputePass<()>,
    generate_scattering_lut_pass: CameraComputePass<()>,
    generate_sky_lut_pass: CameraComputePass<()>,

    is_initialized: Mutex<bool>,
    known_sun_altitude: Mutex<Option<f32>>,
}

impl AtmospherePass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let generate_transmittance_lut_pass =
            CameraComputePass::builder("atmosphere_generate_transmittance_lut")
                .bind([&buffers.atmosphere_transmittance_lut.bind_writable()])
                .with_entry_point("main_generate_transmittance_lut")
                .build(device, &engine.shaders.atmosphere);

        let generate_scattering_lut_pass =
            CameraComputePass::builder("atmosphere_generate_scattering_lut")
                .bind([
                    &buffers.atmosphere_transmittance_lut.bind_sampled(),
                    &buffers.atmosphere_scattering_lut.bind_writable(),
                ])
                .with_entry_point("main_generate_scattering_lut")
                .build(device, &engine.shaders.atmosphere);

        let generate_sky_lut_pass =
            CameraComputePass::builder("atmosphere_generate_sky_lut")
                .bind([
                    &engine.world.bind_readable(),
                    &buffers.atmosphere_transmittance_lut.bind_sampled(),
                    &buffers.atmosphere_scattering_lut.bind_sampled(),
                    &buffers.atmosphere_sky_lut.bind_writable(),
                ])
                .with_entry_point("main_generate_sky_lut")
                .build(device, &engine.shaders.atmosphere);

        Self {
            generate_transmittance_lut_pass,
            generate_scattering_lut_pass,
            generate_sky_lut_pass,

            is_initialized: Mutex::new(false),
            known_sun_altitude: Mutex::new(None),
        }
    }

    pub fn run<P>(
        &self,
        engine: &Engine<P>,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        P: Params,
    {
        let mut is_initialized = self.is_initialized.lock().unwrap();
        let mut known_sun_altitude = self.known_sun_altitude.lock().unwrap();

        // Transmittance and scattering don't depend on anything so it's enough
        // if we just generate them once, the first time they are needed:
        if !*is_initialized {
            self.generate_transmittance_lut_pass.run(
                camera,
                encoder,
                gpu::Atmosphere::TRANSMITTANCE_LUT_RESOLUTION / 8,
                &(),
            );

            self.generate_scattering_lut_pass.run(
                camera,
                encoder,
                gpu::Atmosphere::SCATTERING_LUT_RESOLUTION / 8,
                &(),
            );

            *is_initialized = true;
        }

        // On the other hand, the sky lookup texture depends on sun's position:
        if known_sun_altitude
            .map_or(true, |altitude| altitude != engine.sun.altitude)
        {
            self.generate_sky_lut_pass.run(
                camera,
                encoder,
                gpu::Atmosphere::SKY_LUT_RESOLUTION / 8,
                &(),
            );

            *known_sun_altitude = Some(engine.sun.altitude);
        }
    }
}
