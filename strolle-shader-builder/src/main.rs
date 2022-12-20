use std::{env, fs};

use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

const SHADERS: &[&str] = &["materializer", "printer", "tracer"];

fn main() {
    // HACK Normally, when compiling shaders, spirv-builder uses the regular
    //      `target` directory for the results - this poses an inconvenience
    //      when you alternately build shaders and examples (e.g. during
    //      development), because building shaders discards examples' artifacts.
    //
    //      So if you build shaders and then try to run an example, it will
    //      try to rebuild like a hundred of different crates instead of just
    //      the ones in our workspace.
    //
    //      Setting those env-vars mitigates this issue, since it simulates a
    //      nested Cargo invocation, which spirv-builder detects and tries to
    //      alleviate on its own, using `--target-dir` - and this fixes the
    //      "artifacts getting randomly invalidated" problem.
    env::set_var("PROFILE", "release");
    env::set_var("OUT_DIR", "../target/spirv/release/build/shader/out");

    for shader in SHADERS {
        let compile_result = SpirvBuilder::new(
            format!("strolle-{shader}"),
            "spirv-unknown-spv1.5",
        )
        .print_metadata(MetadataPrintout::None)
        .capability(Capability::Int8)
        .release(true)
        .build()
        .unwrap();

        fs::copy(
            compile_result.module.unwrap_single(),
            format!("target/{shader}.spv"),
        )
        .unwrap();
    }
}
