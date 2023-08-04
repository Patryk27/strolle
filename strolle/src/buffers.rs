mod bind_group;
mod bindable;
mod bufferable;
mod double_buffered;
mod mapped_storage_buffer;
mod mapped_uniform_buffer;
mod storage_buffer;
mod texture;
mod utils;

pub use self::bind_group::*;
pub use self::bindable::*;
pub use self::bufferable::*;
pub use self::double_buffered::*;
pub use self::mapped_storage_buffer::*;
pub use self::mapped_uniform_buffer::*;
pub use self::storage_buffer::*;
pub use self::texture::*;

#[must_use = "buffer might have gotten reallocated which you should probably react upon"]
#[derive(Clone, Copy, Debug, Default)]
pub struct BufferFlushOutcome {
    pub reallocated: bool,
}
