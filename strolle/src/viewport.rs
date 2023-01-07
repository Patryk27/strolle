mod materializer_pass;
mod printer_pass;
mod tracer_pass;

use std::mem;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, Weak};

use spirv_std::glam::UVec2;
use strolle_models as gpu;

use self::materializer_pass::MaterializerPass;
use self::printer_pass::PrinterPass;
use self::tracer_pass::TracerPass;
use crate::buffers::{StorageBuffer, Texture, UniformBuffer};
use crate::{Engine, Params};

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

        let camera = UniformBuffer::new(device, "strolle_camera", camera);

        let hits = StorageBuffer::new_default(
            device,
            "strolle_hits",
            (2 * size.x * size.y) as usize * mem::size_of::<u32>(),
        );

        let image = Texture::new(device, "strolle_image", size);

        let tracer = TracerPass::new(engine, device, &camera, &hits);

        let materializer =
            MaterializerPass::new(engine, device, &camera, &hits, &image);

        let printer = PrinterPass::new(engine, device, format, &camera, &image);

        Self {
            inner: Arc::new(Mutex::new(ViewportInner {
                pos,
                size,
                format,
                camera,
                hits,
                image,
                tracer,
                materializer,
                printer,
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

    pub(crate) fn on_images_changed<P>(
        &self,
        engine: &Engine<P>,
        device: &wgpu::Device,
    ) where
        P: Params,
    {
        log::debug!("Images changed - rebuilding pipelines");

        self.with(|this| {
            this.tracer =
                TracerPass::new(engine, device, &this.camera, &this.hits);

            this.materializer = MaterializerPass::new(
                engine,
                device,
                &this.camera,
                &this.hits,
                &this.image,
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
            this.tracer.run(this.size, encoder);
            this.materializer.run(this.size, encoder);
            this.printer.run(this.pos, this.size, encoder, target);
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

pub(crate) struct WeakViewport {
    inner: Weak<Mutex<ViewportInner>>,
}

impl WeakViewport {
    pub fn upgrade(&self) -> Option<Viewport> {
        self.inner.upgrade().map(|inner| Viewport { inner })
    }
}

struct ViewportInner {
    pos: UVec2,
    size: UVec2,
    format: wgpu::TextureFormat,
    camera: UniformBuffer<gpu::Camera>,
    hits: StorageBuffer<u32>,
    image: Texture,
    tracer: TracerPass,
    materializer: MaterializerPass,
    printer: PrinterPass,
}

impl Drop for ViewportInner {
    fn drop(&mut self) {
        log::info!(
            "Releasing viewport ({})",
            Viewport::describe(self.pos, self.size, self.format)
        );
    }
}
