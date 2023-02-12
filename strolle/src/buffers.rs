mod descriptor_set;
mod mapped_storage_buffer;
mod mapped_uniform_buffer;
mod texture;
mod unmapped_storage_buffer;

pub use self::descriptor_set::*;
pub use self::mapped_storage_buffer::*;
pub use self::mapped_uniform_buffer::*;
pub use self::texture::*;
pub use self::unmapped_storage_buffer::*;

pub trait Bindable {
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, wgpu::BindingResource)>;
}
