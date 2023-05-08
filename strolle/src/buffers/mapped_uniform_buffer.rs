use std::mem;
use std::ops::{Deref, DerefMut};

use log::info;

use crate::buffers::utils;
use crate::{Bindable, Bufferable};

#[derive(Debug)]
pub struct MappedUniformBuffer<T> {
    buffer: wgpu::Buffer,
    data: T,
    dirty: bool,
}

impl<T> MappedUniformBuffer<T>
where
    T: Bufferable,
{
    pub fn new(device: &wgpu::Device, label: impl AsRef<str>, data: T) -> Self {
        let label = label.as_ref();
        let size = utils::pad_size(data.size());

        info!("Allocating uniform buffer `{label}`; size={size}");

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
