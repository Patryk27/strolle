mod descriptor_set;
mod storage_buffer;
mod uniform_buffer;

pub use self::descriptor_set::*;
pub use self::storage_buffer::*;
pub use self::uniform_buffer::*;

pub trait Bufferable {
    fn layout(
        &self,
        binding: u32,
    ) -> (wgpu::BindingResource, wgpu::BindGroupLayoutEntry);
}
