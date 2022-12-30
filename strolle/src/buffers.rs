mod descriptor_set;
mod storage_buffer;
mod texture;
mod uniform_buffer;

pub use self::descriptor_set::*;
pub use self::storage_buffer::*;
pub use self::texture::*;
pub use self::uniform_buffer::*;

pub trait Bindable {
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, wgpu::BindingResource)>;
}
