use std::fs;

use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

fn main() {
    let compile_result =
        SpirvBuilder::new("strolle-shader", "spirv-unknown-spv1.5")
            .print_metadata(MetadataPrintout::None)
            .capability(Capability::Int8)
            .release(true)
            .build()
            .unwrap();

    fs::copy(compile_result.module.unwrap_single(), "target/shader.spv")
        .unwrap();
}
