use std::marker::PhantomData;
use std::{any, mem, slice};

use bytemuck::Pod;

pub struct AllocatedBuffer<T> {
    buffer: wgpu::Buffer,
    _marker: PhantomData<T>,
}

impl<T> AllocatedBuffer<T>
where
    T: Pod,
{
    pub fn create(
        device: &wgpu::Device,
        label: impl AsRef<str>,
    ) -> Option<Self> {
        if mem::size_of::<T>() == 0 {
            return None;
        }

        let size = mem::size_of::<T>();
        let size = (size + 31) & !31;

        log::debug!(
            "Allocating buffer `{}`; size={} (padded to {})",
            any::type_name::<T>(),
            mem::size_of::<T>(),
            size,
        );

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: size as _,
            mapped_at_creation: false,
        });

        Some(Self {
            buffer,
            _marker: PhantomData,
        })
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }

    pub fn write(&self, queue: &wgpu::Queue, data: &T) {
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(slice::from_ref(data)),
        );
    }
}
