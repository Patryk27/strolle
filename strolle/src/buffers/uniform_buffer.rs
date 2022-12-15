use std::marker::PhantomData;
use std::{any, mem, slice};

use bytemuck::Pod;

use super::Bindable;

pub struct UniformBuffer<T> {
    buffer: wgpu::Buffer,
    _marker: PhantomData<T>,
}

impl<T> UniformBuffer<T>
where
    T: Pod,
{
    pub fn new(device: &wgpu::Device, label: impl AsRef<str>) -> Self {
        let label = label.as_ref();
        let size = mem::size_of::<T>();
        let size = (size + 31) & !31;

        log::debug!(
            "Allocating uniform buffer `{label}`; ty={}, size={size} (padded to {})",
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
            _marker: PhantomData,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue, data: &T) {
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(slice::from_ref(data)),
        );
    }
}

impl<T> Bindable for UniformBuffer<T> {
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
