use crate::{Bindable, DoubleBufferedBindable, Texture, TextureBuilder};

#[derive(Debug)]
pub struct DoubleBuffered<T> {
    a: T,
    b: T,
}

impl DoubleBuffered<Texture> {
    /// Creates a double-buffered texture.
    ///
    /// See: [`Texture::new()`].
    pub fn new(device: &wgpu::Device, texture: TextureBuilder) -> Self {
        let label_a = format!("{}_a", texture.label());
        let label_b = format!("{}_b", texture.label());

        Self {
            a: texture.clone().with_label(label_a).build(device),
            b: texture.with_label(label_b).build(device),
        }
    }
}

impl DoubleBuffered<&Texture> {
    /// See: [`Texture::bind_readable()`].
    pub fn bind_readable(&self) -> impl DoubleBufferedBindable + '_ {
        DoubleBufferedBinder {
            a: self.a.bind_readable(),
            b: self.b.bind_readable(),
        }
    }

    /// See: [`Texture::bind_writable()`].
    pub fn bind_writable(&self) -> impl DoubleBufferedBindable + '_ {
        DoubleBufferedBinder {
            a: self.a.bind_writable(),
            b: self.b.bind_writable(),
        }
    }
}

impl<T> DoubleBuffered<T> {
    pub fn get(&self, alternate: bool) -> &T {
        if alternate {
            &self.b
        } else {
            &self.a
        }
    }

    pub fn curr(&self) -> DoubleBuffered<&T> {
        DoubleBuffered {
            a: &self.a,
            b: &self.b,
        }
    }

    pub fn prev(&self) -> DoubleBuffered<&T> {
        DoubleBuffered {
            a: &self.b,
            b: &self.a,
        }
    }
}

pub struct DoubleBufferedBinder<T> {
    a: T,
    b: T,
}

impl<T> DoubleBufferedBindable for DoubleBufferedBinder<T>
where
    T: Bindable,
{
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, [wgpu::BindingResource; 2])> {
        let entries_a = self.a.bind(binding);
        let entries_b = self.b.bind(binding);

        assert_eq!(entries_a.len(), entries_b.len());

        entries_a
            .into_iter()
            .zip(entries_b)
            .map(|((layout_a, resource_a), (layout_b, resource_b))| {
                assert_eq!(layout_a, layout_b);

                #[allow(
                    clippy::tuple_array_conversions,
                    reason="https://github.com/rust-lang/rust-clippy/issues/11144"
                )]
                (layout_a, [resource_a, resource_b])
            })
            .collect()
    }
}
