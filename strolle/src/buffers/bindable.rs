/// Object that can be attached to a pipeline, e.g. a buffer or a texture
pub trait Bindable {
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, wgpu::BindingResource)>;
}

/// Object that can be attached to a pipeline, e.g. a buffer or a texture, and
/// it's double-buffered (i.e. exists in two similar versions swapped after each
/// frame)
pub trait DoubleBufferedBindable {
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, [wgpu::BindingResource; 2])>;
}

impl<T> DoubleBufferedBindable for T
where
    T: Bindable,
{
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, [wgpu::BindingResource; 2])> {
        T::bind(self, binding)
            .into_iter()
            .map(|(layout, resource)| {
                let resource_a = resource.clone();
                let resource_b = resource;

                (layout, [resource_a, resource_b])
            })
            .collect()
    }
}
