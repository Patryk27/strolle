use std::ops::{Deref, DerefMut};
use std::{any, mem, slice};

use bytemuck::Pod;

use super::Bindable;

/// Storage buffer that exists both on the host machine and the GPU.
///
/// This kind of storage buffer should be used for data structures such as BVH
/// that need to be accessed both from the host machine and the GPU; it's
/// allocated both in RAM and VRAM, and uses [`DerefMut`] to track whether it's
/// been modified recently.
///
/// TODO support buffers with dynamic length
#[derive(Debug)]
pub struct MappedStorageBuffer<T> {
    buffer: wgpu::Buffer,
    data: T,
    dirty: bool,
}

impl<T> MappedStorageBuffer<T>
where
    T: StorageBufferable,
{
    pub fn new(
        device: &wgpu::Device,
        label: impl AsRef<str>,
        size: usize,
        data: T,
    ) -> Self {
        let label = label.as_ref();

        log::info!(
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
            data,
            dirty: true,
        }
    }

    pub fn new_default(
        device: &wgpu::Device,
        label: impl AsRef<str>,
        size: usize,
    ) -> Self
    where
        T: Default,
    {
        Self::new(device, label, size, Default::default())
    }

    pub fn flush(&mut self, queue: &wgpu::Queue) {
        if !mem::take(&mut self.dirty) {
            return;
        }

        queue.write_buffer(&self.buffer, 0, self.data.data());
    }

    pub fn flush_ex(
        &mut self,
        queue: &wgpu::Queue,
        offset: usize,
        size: usize,
    ) {
        queue.write_buffer(
            &self.buffer,
            offset as _,
            &self.data.data()[offset..][..size],
        );
    }
}

impl<T> Deref for MappedStorageBuffer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for MappedStorageBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty = true;

        &mut self.data
    }
}

impl<T> Bindable for MappedStorageBuffer<T> {
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
                    //      wgpu to reject the shader later
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

impl StorageBufferable for u64 {
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
