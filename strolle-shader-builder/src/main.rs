use std::env;
use std::error::Error;
use std::path::Path;

use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    let crate_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("strolle-shaders");

    let result = SpirvBuilder::new(crate_path, "spirv-unknown-spv1.3")
        .multimodule(true)
        .print_metadata(MetadataPrintout::DependencyOnly)
        .capability(Capability::Int8)
        .extra_arg("--spirt-passes=reduce,fuse_selects")
        .build()?;

    for (shader_name, shader_path) in result.module.unwrap_multi() {
        let shader_id = shader_name.replace("::", "_");
        let shader_id = shader_id.strip_suffix("_main").unwrap_or(&shader_id);

        println!(
            "cargo:rustc-env=strolle_shaders::{}.path={}",
            shader_id,
            shader_path.display()
        );

        println!(
            "cargo:rustc-env=strolle_shaders::{}.entry_point={}",
            shader_id, shader_name,
        );
    }

    Ok(())
}
