use std::collections::HashMap;

use glam::{uvec2, vec3, UVec2, Vec3, Vec3Swizzles};
use image::{ImageBuffer, Rgb};
use strolle_gpu::Noise;

const RESOLUTION: usize = 1024;

fn main() {
    let mut samples: HashMap<_, usize> = Default::default();

    for seed in 0..10_000_000 {
        let mut noise = Noise::new(seed, uvec2(234, 345));
        let sample = noise.sample_hemisphere(vec3(0.0, 0.0, 1.0));

        assert!(
            sample.length() >= 0.99 && sample.length() <= 1.01,
            "Sample has invalid length: {sample:?} (={})",
            sample.length()
        );

        let sample = Vec3::splat(RESOLUTION as f32 / 2.0)
            + sample * (RESOLUTION as f32 / 2.0);

        assert!(
            sample.x >= 0.0
                && sample.y >= 0.0
                && sample.z >= 0.0
                && sample.x <= 1024.0
                && sample.y <= 1024.0
                && sample.z <= 1024.0,
            "Sample out of range: {sample:?}"
        );

        let sample = sample
            .xy()
            .as_uvec2()
            .min(UVec2::splat((RESOLUTION - 1) as _));

        *samples.entry(sample).or_default() += 1;
    }

    println!("unique samples: {}", samples.len());

    // ---

    let mut image =
        ImageBuffer::<Rgb<u8>, _>::new(RESOLUTION as _, RESOLUTION as _);

    let max_sample_count = *samples.values().max().unwrap();

    for (sample, sample_count) in &samples {
        let color = (255 * *sample_count / max_sample_count) as u8;

        image.get_pixel_mut(sample.x, sample.y).0 = [color, color, color];
    }

    image.save("output.png").unwrap();
}
