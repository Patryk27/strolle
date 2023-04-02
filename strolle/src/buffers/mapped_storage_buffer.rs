use std::ops::{Deref, DerefMut};
use std::{any, mem};

use super::{Bindable, Bufferable};

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
    T: Bufferable,
{
    pub fn new(
        device: &wgpu::Device,
        label: impl AsRef<str>,
        size: usize,
        data: T,
    ) -> Self {
        let label = label.as_ref();
        let size = (size + 31) & !31;

        log::info!(
            "Allocating storage buffer `{label}`; ty={}, size={size}",
            any::type_name::<T>(),
        );

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX,
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

    pub fn as_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn as_ro_bind(&self) -> impl Bindable + '_ {
        MappedStorageBufferBinder {
            parent: self,
            read_only: true,
        }
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

pub struct MappedStorageBufferBinder<'a, T> {
    parent: &'a MappedStorageBuffer<T>,
    read_only: bool,
}

impl<T> Bindable for MappedStorageBufferBinder<'_, T> {
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
