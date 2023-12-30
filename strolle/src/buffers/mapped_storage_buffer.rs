use std::mem;
use std::ops::{Deref, DerefMut};

use log::debug;

use crate::buffers::utils;
use crate::{Bindable, BufferFlushOutcome, Bufferable};

/// Storage buffer that exists both in RAM and VRAM.
///
/// This kind of buffer should be used for data structures such as BVH that need
/// to be accessed both from the host machine and the GPU; it's allocated both
/// in RAM and VRAM, and uses [`DerefMut`] to track whether it's been modified
/// and needs to be flushed.
#[derive(Debug)]
pub struct MappedStorageBuffer<T> {
    label: String,
    buffer: wgpu::Buffer,
    data: T,
    dirty: bool,
}

impl<T> MappedStorageBuffer<T>
where
    T: Bufferable,
{
    pub fn new(device: &wgpu::Device, label: impl AsRef<str>, data: T) -> Self {
        let label = format!("strolle_{}", label.as_ref());

        let size = if data.size() == 0 {
            // If the buffer is empty - just like triangles or initial BVH - it
            // is easier to pretend the buffer is just small instead of
            // zero-sized so that we can allocate *something* here and let the
            // reallocation logic worry about growing the buffer later.
            //
            // That is, since we can't really allocate an empty buffer, the
            // other solution would be to keep `buffer: Option<wgpu::Buffer>`
            // and allocate it on-demand on the first write, and that is just
            // more trouble than it's worth.
            128 * 1024
        } else {
            utils::pad_size(data.size())
        };

        debug!("Allocating mapped storage buffer `{label}`; size={size}");

        let buffer = Self::create_buffer(device, &label, size);

        Self {
            label,
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

    pub fn as_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Creates an immutable storage-buffer binding:
    ///
    /// ```
    /// #[spirv(descriptor_set = ..., binding = ..., storage_buffer)]
    /// items: &[T],
    /// ```
    pub fn bind_readable(&self) -> impl Bindable + '_ {
        MappedStorageBufferBinder {
            parent: self,
            read_only: true,
        }
    }

    pub fn reallocate(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> bool {
        let prev_size = self.buffer.size() as usize;
        let curr_size = utils::pad_size(self.data.size());

        if (self.buffer.size() as usize) >= curr_size {
            return false;
        }

        // TODO consider better strategy
        let target_size = 2 * curr_size;

        debug!(
            "Reallocating mapped storage buffer `{}`; \
             prev-size={prev_size}, curr-size={curr_size}, target-size={target_size}",
            self.label,
        );

        self.buffer.destroy();
        self.buffer = Self::create_buffer(device, &self.label, target_size);
        self.dirty = false;

        queue.write_buffer(&self.buffer, 0, self.data.data());

        true
    }

    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> BufferFlushOutcome {
        if !mem::take(&mut self.dirty) {
            return BufferFlushOutcome::default();
        }

        let reallocated = self.reallocate(device, queue);

        if reallocated {
            // Reallocating already flushes the entire buffer, so there's no
            // need to flush it again
        } else {
            queue.write_buffer(&self.buffer, 0, self.data.data());
        }

        BufferFlushOutcome { reallocated }
    }

    pub fn flush_part(
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

    fn create_buffer(
        device: &wgpu::Device,
        label: &str,
        size: usize,
    ) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX,
            size: size as _,
            mapped_at_creation: false,
        })
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
