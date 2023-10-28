use log::debug;

use crate::buffers::utils;
use crate::Bindable;

/// Storage buffer that exists only in VRAM.
///
/// This kind of storage buffer should be used for data structures that don't
/// have to be accessed on the host machine.
#[derive(Debug)]
pub struct StorageBuffer {
    buffer: wgpu::Buffer,
}

impl StorageBuffer {
    // TODO provide `::builder()` pattern
    pub fn new(
        device: &wgpu::Device,
        label: impl AsRef<str>,
        size: usize,
    ) -> Self {
        let label = label.as_ref();
        let size = utils::pad_size(size);

        debug!("Allocating storage buffer `{label}`; size={size}");

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            usage: wgpu::BufferUsages::STORAGE,
            size: size as _,
            mapped_at_creation: false,
        });

        Self { buffer }
    }

    /// Creates an immutable storage-buffer binding:
    ///
    /// ```
    /// #[spirv(descriptor_set = ..., binding = ..., storage_buffer)]
    /// items: &[T],
    /// ```
    pub fn bind_readable(&self) -> impl Bindable + '_ {
        StorageBufferBinder {
            parent: self,
            read_only: true,
        }
    }

    /// Creates a mutable storage-buffer binding:
    ///
    /// ```
    /// #[spirv(descriptor_set = ..., binding = ..., storage_buffer)]
    /// items: &mut [T],
    /// ```
    pub fn bind_writable(&self) -> impl Bindable + '_ {
        StorageBufferBinder {
            parent: self,
            read_only: false,
        }
    }
}

pub struct StorageBufferBinder<'a> {
    parent: &'a StorageBuffer,
    read_only: bool,
}

impl Bindable for StorageBufferBinder<'_> {
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, wgpu::BindingResource)> {
        let layout = wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::all(),
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage {
                    read_only: self.read_only,
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let resource = self.parent.buffer.as_entire_binding();

        vec![(layout, resource)]
    }
}
