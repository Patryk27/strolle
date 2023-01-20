use super::Bindable;

/// Storage buffer that exists only on the GPU.
///
/// This kind of storage buffer should be used for data structures that don't
/// have to be written / accessed on the host machine, because it doesn't cause
/// the data to be written to / read from host's RAM.
#[derive(Debug)]
pub struct UnmappedStorageBuffer {
    buffer: wgpu::Buffer,
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
            usage: wgpu::BufferUsages::STORAGE,
            size: size as _,
            mapped_at_creation: false,
        });

        Self { buffer }
    }
}

impl Bindable for UnmappedStorageBuffer {
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
