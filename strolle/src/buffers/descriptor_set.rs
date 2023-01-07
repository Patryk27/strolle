use super::Bindable;

pub struct DescriptorSet {
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl DescriptorSet {
    pub fn builder<'ctx>(name: &str) -> DescriptorSetBuilder<'_, 'ctx> {
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
    layouts: Vec<wgpu::BindGroupLayoutEntry>,
    resources: Vec<wgpu::BindingResource<'ctx>>,
}

impl<'name, 'ctx> DescriptorSetBuilder<'name, 'ctx> {
    fn new(name: &'name str) -> Self {
        Self {
            name,
            layouts: Default::default(),
            resources: Default::default(),
        }
    }

    pub fn add(mut self, item: &'ctx dyn Bindable) -> Self {
        for (layout, resource) in item.bind(self.resources.len() as _) {
            self.layouts.push(layout);
            self.resources.push(resource);
        }

        self
    }

    pub fn build(self, device: &wgpu::Device) -> DescriptorSet {
        let name = self.name;

        let bind_group_layout = {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{name}_layout")),
                entries: &self.layouts,
            })
        };

        let bind_group = {
            let entries: Vec<_> = self
                .resources
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
