use std::marker::PhantomData;
use std::{any, slice};

use bytemuck::Pod;

use super::Bindable;

pub struct StorageBuffer<T> {
    buffer: wgpu::Buffer,
    _marker: PhantomData<T>,
}

impl<T> StorageBuffer<T>
where
    T: StorageBufferable,
{
    pub fn new(
        device: &wgpu::Device,
        label: impl AsRef<str>,
        size: usize,
    ) -> Self {
        let label = label.as_ref();

        log::debug!(
            "Allocating storage buffer `{label}`; ty={}, size={size}",
            any::type_name::<T>(),
        );

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            size: size as _,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            _marker: PhantomData,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue, data: &T) {
        queue.write_buffer(&self.buffer, 0, data.data());
    }
}

impl<T> Bindable for StorageBuffer<T> {
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
                    // TODO should say `read_only: true`, but rust-gpu is not
                    //      able to emit appropriate attributes yet, causing
                    //      naga to reject the shader later
                    read_only: false,
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let resource = self.buffer.as_entire_binding();

        vec![(layout, resource)]
    }
}

pub trait StorageBufferable {
    fn data(&self) -> &[u8];
}

impl StorageBufferable for u32 {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(slice::from_ref(self))
    }
}

impl StorageBufferable for f32 {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(slice::from_ref(self))
    }
}

impl<T> StorageBufferable for Vec<T>
where
    T: Pod,
{
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
}
