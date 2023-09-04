use std::env;
use std::error::Error;
use std::path::Path;

use spirv_builder::{Capability, SpirvBuilder};

const CRATES: &[&str] = &[
    "atmosphere",
    "bvh-heatmap",
    "direct-denoising",
    "direct-raster",
    "direct-resolving",
    "direct-shading",
    "direct-spatial-resampling",
    "direct-temporal-resampling",
    "direct-validation",
    "frame-composition",
    "frame-reprojection",
    "indirect-diffuse-denoising",
    "indirect-diffuse-resolving",
    "indirect-diffuse-spatial-resampling",
    "indirect-diffuse-temporal-resampling",
    "indirect-shading",
    "indirect-specular-denoising",
    "indirect-specular-resampling",
    "indirect-specular-resolving",
    "indirect-tracing",
    "reference-shading",
    "reference-tracing",
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
