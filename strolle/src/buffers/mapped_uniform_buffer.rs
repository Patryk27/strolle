use std::ops::{Deref, DerefMut};
use std::{any, mem, slice};

use bytemuck::Pod;

use super::Bindable;

#[derive(Debug)]
pub struct MappedUniformBuffer<T> {
    buffer: wgpu::Buffer,
    data: T,
    dirty: bool,
}

impl<T> MappedUniformBuffer<T>
where
    T: UniformBufferable,
{
    pub fn new(device: &wgpu::Device, label: impl AsRef<str>, data: T) -> Self {
        let label = label.as_ref();
        let size = mem::size_of::<T>();
        let size = (size + 31) & !31;

        log::info!(
            "Allocating uniform buffer `{label}`; ty={}, size={size} (padded from {})",
            any::type_name::<T>(),
            mem::size_of::<T>(),
        );

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            size: size as _,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            data,
            dirty: true,
        }
    }

    pub fn new_default(device: &wgpu::Device, label: impl AsRef<str>) -> Self
    where
        T: Default,
    {
        Self::new(device, label, Default::default())
    }

    pub fn flush(&mut self, queue: &wgpu::Queue) {
        if !mem::take(&mut self.dirty) {
            return;
        }

        queue.write_buffer(&self.buffer, 0, self.data.data());
    }
}

impl<T> Deref for MappedUniformBuffer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for MappedUniformBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty = true;

        &mut self.data
    }
}

impl<T> Bindable for MappedUniformBuffer<T> {
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, wgpu::BindingResource)> {
        let layout = wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT
                | wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let resource = self.buffer.as_entire_binding();

        vec![(layout, resource)]
    }
}

pub trait UniformBufferable {
    fn size(&self) -> usize;
    fn data(&self) -> &[u8];
}

impl<T> UniformBufferable for T
where
    T: Pod,
{
    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }

    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(slice::from_ref(self))
    }
}
