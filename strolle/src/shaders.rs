pub struct Shaders {
    pub printing_pass: wgpu::ShaderModule,
    pub raygen_pass: wgpu::ShaderModule,
    pub shading_pass: wgpu::ShaderModule,
    pub tracing_pass: wgpu::ShaderModule,
}

impl Shaders {
    pub fn new(device: &wgpu::Device) -> Self {
        let printing_pass = device.create_shader_module(wgpu::include_spirv!(
            "../../target/printing-pass.spv"
        ));

        let raygen_pass = device.create_shader_module(wgpu::include_spirv!(
            "../../target/raygen-pass.spv"
        ));

        let shading_pass = device.create_shader_module(wgpu::include_spirv!(
            "../../target/shading-pass.spv"
        ));

        let tracing_pass = device.create_shader_module(wgpu::include_spirv!(
            "../../target/tracing-pass.spv"
        ));

        Self {
            printing_pass,
            raygen_pass,
            shading_pass,
            tracing_pass,
        }
    }
}
