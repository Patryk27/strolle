#[derive(Debug)]
pub struct Shaders {
    pub drawing_pass: wgpu::ShaderModule,
    pub ray_shading_pass: wgpu::ShaderModule,
    pub ray_tracing_pass: wgpu::ShaderModule,
}

impl Shaders {
    pub fn new(device: &wgpu::Device) -> Self {
        let drawing_pass = device.create_shader_module(wgpu::include_spirv!(
            "../../target/pass-drawing.spv"
        ));

        let ray_shading_pass = device.create_shader_module(
            wgpu::include_spirv!("../../target/pass-ray-shading.spv"),
        );

        let ray_tracing_pass = device.create_shader_module(
            wgpu::include_spirv!("../../target/pass-ray-tracing.spv"),
        );

        Self {
            drawing_pass,
            ray_shading_pass,
            ray_tracing_pass,
        }
    }
}
