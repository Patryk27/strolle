mod drawing_pass;
mod ray_shading_pass;
mod ray_tracing_pass;

use std::mem;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, Weak};

use rand::Rng;
use spirv_std::glam::{UVec2, Vec4};
use strolle_models as gpu;

use self::drawing_pass::*;
use self::ray_shading_pass::*;
use self::ray_tracing_pass::*;
use crate::buffers::{MappedUniformBuffer, Texture, UnmappedStorageBuffer};
use crate::{Engine, Params};

#[derive(Debug)]
pub struct Viewport {
    inner: Arc<Mutex<ViewportInner>>,
}

impl Viewport {
    pub(crate) fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        pos: UVec2,
        size: UVec2,
        format: wgpu::TextureFormat,
        camera: gpu::Camera,
    ) -> Self
    where
        P: Params,
    {
        log::info!(
            "Creating viewport ({})",
            Viewport::describe(pos, size, format)
        );

        assert!(size.x > 0);
        assert!(size.y > 0);

        let camera = MappedUniformBuffer::new(device, "strolle_camera", camera);

        let ray_origins = UnmappedStorageBuffer::new(
            device,
            "strolle_ray_origins",
            (size.x * size.y) as usize * mem::size_of::<Vec4>(),
        );

        let ray_directions = UnmappedStorageBuffer::new(
            device,
            "strolle_ray_directions",
            (size.x * size.y) as usize * mem::size_of::<Vec4>(),
        );

        let ray_throughputs = UnmappedStorageBuffer::new(
            device,
            "strolle_ray_throughputs",
            (size.x * size.y) as usize * mem::size_of::<Vec4>(),
        );

        let ray_hits = UnmappedStorageBuffer::new(
            device,
            "strolle_ray_hits",
            (2 * size.x * size.y) as usize * mem::size_of::<Vec4>(),
        );

        let colors = Texture::new(
            device,
            "strolle_colors",
            size,
            wgpu::TextureFormat::Rgba16Float,
        );

        let normals = Texture::new(
            device,
            "strolle_normals",
            size,
            wgpu::TextureFormat::Rgba16Float,
        );

        let bvh_heatmap = Texture::new(
            device,
            "strolle_bvh_heatmap",
            size,
            wgpu::TextureFormat::Rgba8Unorm,
        );

        let drawing_pass = DrawingPass::new(
            engine,
            device,
            format,
            &camera,
            &colors,
            &normals,
            &bvh_heatmap,
        );

        let ray_shading_pass = RayShadingPass::new(
            engine,
            device,
            &camera,
            &ray_origins,
            &ray_directions,
            &ray_throughputs,
            &ray_hits,
            &colors,
            &normals,
            &bvh_heatmap,
        );

        let ray_tracing_pass = RayTracingPass::new(
            engine,
            device,
            &camera,
            &ray_origins,
            &ray_directions,
            &ray_hits,
        );

        Self {
            inner: Arc::new(Mutex::new(ViewportInner {
                pos,
                size,
                format,

                camera,
                ray_origins,
                ray_directions,
                ray_throughputs,
                ray_hits,
                colors,
                normals,
                bvh_heatmap,

                drawing_pass,
                ray_shading_pass,
                ray_tracing_pass,

                config: Default::default(),
                tick: 0,
            })),
        }
    }

    pub(crate) fn downgrade(&self) -> WeakViewport {
        WeakViewport {
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub fn pos(&self) -> UVec2 {
        self.with(|this| this.pos)
    }

    pub fn size(&self) -> UVec2 {
        self.with(|this| this.size)
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.with(|this| this.format)
    }

    pub fn set_camera(&self, camera: gpu::Camera) {
        self.with(|this| {
            *this.camera.deref_mut() = camera;
        });
    }

    pub fn set_config(&self, config: ViewportConfiguration) {
        self.with(|this| {
            this.config = config;
        });
    }

    pub(crate) fn rebuild<P>(&self, engine: &Engine<P>, device: &wgpu::Device)
    where
        P: Params,
    {
        self.with(|this| {
            log::info!(
                "Rebuilding viewport ({})",
                Self::describe(this.pos, this.size, this.format)
            );

            this.drawing_pass = DrawingPass::new(
                engine,
                device,
                this.format,
                &this.camera,
                &this.colors,
                &this.normals,
                &this.bvh_heatmap,
            );

            this.ray_shading_pass = RayShadingPass::new(
                engine,
                device,
                &this.camera,
                &this.ray_origins,
                &this.ray_directions,
                &this.ray_throughputs,
                &this.ray_hits,
                &this.colors,
                &this.normals,
                &this.bvh_heatmap,
            );

            this.ray_tracing_pass = RayTracingPass::new(
                engine,
                device,
                &this.camera,
                &this.ray_origins,
                &this.ray_directions,
                &this.ray_hits,
            );
        });
    }

    pub fn flush(&self, queue: &wgpu::Queue) {
        self.with(|this| {
            this.camera.flush(queue);
        });
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        self.with(|this| {
            this.tick += 1;

            for bounce in 0..=this.config.bounces() {
                let params = gpu::RayPassParams {
                    bounce: bounce as u32,
                    seed: rand::thread_rng().gen(),
                    tick: this.tick,
                    apply_denoising: this.config.apply_denoising() as u32,
                };

                this.ray_tracing_pass.run(this.size, params, encoder);
                this.ray_shading_pass.run(this.size, params, encoder);
            }

            let params = gpu::DrawingPassParams {
                viewport_mode: this.config.mode.serialize(),
            };

            this.drawing_pass
                .run(this.pos, this.size, params, encoder, target);
        });
    }

    fn with<T>(&self, f: impl FnOnce(&mut ViewportInner) -> T) -> T {
        f(&mut self.inner.lock().unwrap())
    }

    fn describe(
        pos: UVec2,
        size: UVec2,
        format: wgpu::TextureFormat,
    ) -> String {
        format!(
            "pos={}x{}, size={}x{}, format={:?}",
            pos.x, pos.y, size.x, size.y, format
        )
    }
}

#[derive(Debug)]
pub(crate) struct WeakViewport {
    inner: Weak<Mutex<ViewportInner>>,
}

impl WeakViewport {
    pub fn upgrade(&self) -> Option<Viewport> {
        self.inner.upgrade().map(|inner| Viewport { inner })
    }
}

#[derive(Debug)]
struct ViewportInner {
    pos: UVec2,
    size: UVec2,
    format: wgpu::TextureFormat,

    camera: MappedUniformBuffer<gpu::Camera>,
    ray_origins: UnmappedStorageBuffer,
    ray_directions: UnmappedStorageBuffer,
    ray_throughputs: UnmappedStorageBuffer,
    ray_hits: UnmappedStorageBuffer,
    colors: Texture,
    normals: Texture,
    bvh_heatmap: Texture,

    drawing_pass: DrawingPass,
    ray_shading_pass: RayShadingPass,
    ray_tracing_pass: RayTracingPass,

    config: ViewportConfiguration,
    tick: u32,
}

impl Drop for ViewportInner {
    fn drop(&mut self) {
        log::info!(
            "Releasing viewport ({})",
            Viewport::describe(self.pos, self.size, self.format)
        );
    }
}

#[derive(Clone, Debug, Default)]
pub struct ViewportConfiguration {
    pub mode: ViewportMode,
    pub bounces: usize,
}

impl ViewportConfiguration {
    fn bounces(&self) -> usize {
        if self.mode == ViewportMode::DisplayImage {
            self.bounces
        } else {
            // If we're displaying normals and/or the heatmap, there's no need
            // to trace any bounces, because they don't incorporate any more
            // detail into those modes.
            //
            // (that is to say, the normal-output will look the same for zero
            // bounces as it would for ten.)
            0
        }
    }

    /// Returns whether the final image should be denoised.
    ///
    /// Currently it's kind of a hack to avoid denoising when we're running in
    /// the ray-tracing mode (i.e. without tracing bounces), since otherwise it
    /// causes some pretty visible artifacts (e.g. on the `cubes` example).
    ///
    /// Overall we need this only because our current denoising algorithm is
    /// bad - we should be able to remove it having migrated to ReSTIR GI or
    /// something else.
    ///
    /// TODO consider removing in the future
    fn apply_denoising(&self) -> bool {
        self.mode == ViewportMode::DisplayImage && self.bounces > 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewportMode {
    DisplayImage,
    DisplayNormals,
    DisplayBvhHeatmap,
}

impl ViewportMode {
    fn serialize(&self) -> u32 {
        match self {
            ViewportMode::DisplayImage => 0,
            ViewportMode::DisplayNormals => 1,
            ViewportMode::DisplayBvhHeatmap => 2,
        }
    }
}

impl Default for ViewportMode {
    fn default() -> Self {
        Self::DisplayImage
    }
}
