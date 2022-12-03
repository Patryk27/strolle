use bytemuck::Pod;

use crate::AllocatedBuffer;

pub struct AllocatedUniform<B0, B1 = (), B2 = ()> {
    buffer0: Option<AllocatedBuffer<B0>>,
    buffer1: Option<AllocatedBuffer<B1>>,
    buffer2: Option<AllocatedBuffer<B2>>,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl<B0, B1, B2> AllocatedUniform<B0, B1, B2>
where
    B0: Pod,
    B1: Pod,
    B2: Pod,
{
    pub fn create(device: &wgpu::Device, name: &str) -> Self {
        log::debug!("Allocating uniform `{}`", name);

        let buffer0 =
            AllocatedBuffer::create(device, format!("{}_buffer0", name));

        let buffer1 =
            AllocatedBuffer::create(device, format!("{}_buffer1", name));

        let buffer2 =
            AllocatedBuffer::create(device, format!("{}_buffer2", name));

        if buffer2.is_some() {
            assert!(buffer1.is_some(), "Cannot allocate uniform with binding=2, since binding=1 is not set");
        }

        if buffer1.is_some() {
            assert!(buffer0.is_some(), "Cannot allocate uniform with binding=1, since binding=0 is not set");
        }

        let buffer_bindings: Vec<_> = {
            let buffer0 = buffer0.as_ref().map(|b| b.as_entire_binding());
            let buffer1 = buffer1.as_ref().map(|b| b.as_entire_binding());
            let buffer2 = buffer2.as_ref().map(|b| b.as_entire_binding());

            [buffer0, buffer1, buffer2].into_iter().flatten().collect()
        };

        let bind_group_layout = {
            let entries: Vec<_> = buffer_bindings
                .iter()
                .enumerate()
                .map(|(binding, _)| wgpu::BindGroupLayoutEntry {
                    binding: binding as _,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                })
                .collect();

            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{name}_bind_group_layout")),
                entries: &entries,
            })
        };

        let bind_group = {
            let entries: Vec<_> = buffer_bindings
                .into_iter()
                .enumerate()
                .map(|(binding, resource)| wgpu::BindGroupEntry {
                    binding: binding as _,
                    resource,
                })
                .collect();

            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("{name}_bind_group")),
                layout: &bind_group_layout,
                entries: &entries,
            })
        };

        AllocatedUniform {
            buffer0,
            buffer1,
            buffer2,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn write0(&self, queue: &wgpu::Queue, data: &B0) {
        self.buffer0
            .as_ref()
            .expect("Tried to write to binding=0, which is uninitialized")
            .write(queue, data);
    }

    pub fn write1(&self, queue: &wgpu::Queue, data: &B1) {
        self.buffer1
            .as_ref()
            .expect("Tried to write to binding=1, which is uninitialized")
            .write(queue, data);
    }

    pub fn write2(&self, queue: &wgpu::Queue, data: &B2) {
        self.buffer2
            .as_ref()
            .expect("Tried to write to binding=2, which is uninitialized")
            .write(queue, data);
    }
}
