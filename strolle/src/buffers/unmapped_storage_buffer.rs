use std::sync::Arc;

use super::Bindable;

/// Storage buffer that exists only on the GPU.
///
/// This kind of storage buffer should be used for data structures that don't
/// have to be written / accessed on the host machine, because it doesn't cause
/// the data to be written to / read from host's RAM.
#[derive(Debug)]
pub struct UnmappedStorageBuffer {
    pub buffer: Arc<wgpu::Buffer>,
}

impl UnmappedStorageBuffer {
    pub fn new(
        device: &wgpu::Device,
        label: impl AsRef<str>,
        size: usize,
    ) -> Self {
        let label = label.as_ref();

        log::info!("Allocating unmapped storage buffer `{label}`; size={size}");

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            size: size as _,
            mapped_at_creation: false,
        });

        Self {
            buffer: Arc::new(buffer),
        }
    }

    pub fn as_ro_bind(&self) -> impl Bindable + '_ {
        UnmappedStorageBufferBinder {
            parent: self,
            read_only: true,
        }
    }

    pub fn as_rw_bind(&self) -> impl Bindable + '_ {
        UnmappedStorageBufferBinder {
            parent: self,
            read_only: false,
        }
    }
}

pub struct UnmappedStorageBufferBinder<'a> {
    parent: &'a UnmappedStorageBuffer,
    read_only: bool,
}

impl Bindable for UnmappedStorageBufferBinder<'_> {
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, wgpu::BindingResource)> {
        let layout = wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT
                | wgpu::ShaderStages::COMPUTE,
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
