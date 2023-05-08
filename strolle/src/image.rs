use std::ops::Deref;

/// Object that yields [`wgpu::TextureView`].
///
/// This exists as a separate thing only because Bevy doesn't expose owned
/// texture views directly, but rather through a newtype (and we need an owned
/// object to store it in our hashmaps without incurring borrowing problems).
pub trait ImageTexture {
    fn get(&self) -> &wgpu::TextureView;
}

impl<T> ImageTexture for T
where
    T: Deref<Target = wgpu::TextureView>,
{
    fn get(&self) -> &wgpu::TextureView {
        self
    }
}

/// Object that yields [`wgpu::Sampler`].
///
/// This exists as a separate thing only because Bevy doesn't expose owned
/// samplers directly, but rather through a newtype (and we need an owned object
/// to store it in our hashmaps without incurring borrowing problems).
pub trait ImageSampler {
    fn get(&self) -> &wgpu::Sampler;
}

impl<T> ImageSampler for T
where
    T: Deref<Target = wgpu::Sampler>,
{
    fn get(&self) -> &wgpu::Sampler {
        self
    }
}
