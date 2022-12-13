use super::Bufferable;

pub struct DescriptorSet {
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl DescriptorSet {
    pub fn builder<'name, 'ctx>(
        name: &'name str,
    ) -> DescriptorSetBuilder<'name, 'ctx> {
        DescriptorSetBuilder::new(name)
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}

pub struct DescriptorSetBuilder<'name, 'ctx> {
    name: &'name str,
    bindings: Vec<wgpu::BindingResource<'ctx>>,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl<'name, 'ctx> DescriptorSetBuilder<'name, 'ctx> {
    fn new(name: &'name str) -> Self {
        Self {
            name,
            bindings: Default::default(),
            entries: Default::default(),
        }
    }

    pub fn add(mut self, buffer: &'ctx dyn Bufferable) -> Self {
        let (binding, entry) = buffer.layout(self.bindings.len() as _);

        self.bindings.push(binding);
        self.entries.push(entry);
        self
    }

    pub fn build(self, device: &wgpu::Device) -> DescriptorSet {
        let name = self.name;

        let bind_group_layout = {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{name}_layout")),
                entries: &self.entries,
            })
        };

        let bind_group = {
            let entries: Vec<_> = self
                .bindings
                .into_iter()
                .enumerate()
                .map(|(binding, resource)| wgpu::BindGroupEntry {
                    binding: binding as _,
                    resource,
                })
                .collect();

            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(name),
                layout: &bind_group_layout,
                entries: &entries,
            })
        };

        DescriptorSet {
            bind_group,
            bind_group_layout,
        }
    }
}
