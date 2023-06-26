use std::env;
use std::error::Error;
use std::path::Path;

use spirv_builder::{Capability, SpirvBuilder};

const CRATES: &[&str] = &[
    "atmosphere",
    "direct-denoising",
    "direct-initial-shading",
    "direct-raster",
    "direct-resolving",
    "direct-spatial-resampling",
    "direct-temporal-resampling",
    "direct-tracing",
    "indirect-denoising",
    "indirect-initial-shading",
    "indirect-initial-tracing",
    "indirect-resolving",
    "indirect-spatial-resampling",
    "indirect-temporal-resampling",
    "output-drawing",
    "reprojection",
];

fn main() -> Result<(), Box<dyn Error>> {
    for crate_name in CRATES {
        let crate_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("strolle-shaders")
            .join(crate_name);

        SpirvBuilder::new(crate_path, "spirv-unknown-spv1.3")
            .capability(Capability::Int8)
            .release(true)
            .build()?;
    }

    Ok(())
}
