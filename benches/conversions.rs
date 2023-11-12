use criterion::{black_box, criterion_group, criterion_main, Criterion};
use colcon::{Space, convert_space, expand_gamma, correct_gamma};

fn pixels() -> Box<[f32]> {
    let size = 512;
    let mut result = Vec::<f32>::with_capacity(size * size * 3);
    for x in 1..=size {
        for y in 1..=size {
            let n = (x as f32 / size as f32 / 2.0) + (y as f32 / size as f32 / 2.0);
            result.extend_from_slice(&[n, n, n]);
        }
    }
    result.into_boxed_slice()
}

pub fn conversions(c: &mut Criterion) {
    let pixels = pixels();

    c.bench_function("srgb_to_lrgb", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::SRGB, Space::LRGB, pixel.try_into().unwrap())));
    } ));

    c.bench_function("lrgb_to_xyz", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::LRGB, Space::XYZ, pixel.try_into().unwrap())));
    } ));

    c.bench_function("xyz_to_lab", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::XYZ, Space::LAB, pixel.try_into().unwrap())));
    } ));

    c.bench_function("lab_to_lch", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::LAB, Space::LCH, pixel.try_into().unwrap())));
    } ));

    c.bench_function("lch_to_lab", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::LCH, Space::LAB, pixel.try_into().unwrap())));
    } ));

    c.bench_function("lab_to_xyz", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::LAB, Space::XYZ, pixel.try_into().unwrap())));
    } ));

    c.bench_function("xyz_to_lrgb", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::XYZ, Space::LRGB, pixel.try_into().unwrap())));
    } ));

    c.bench_function("lrgb_to_srgb", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::LRGB, Space::SRGB, pixel.try_into().unwrap())));
    } ));

    c.bench_function("srgb_to_hsv", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::SRGB, Space::HSV, pixel.try_into().unwrap())));
    } ));

    c.bench_function("hsv_to_srgb", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::HSV, Space::SRGB, pixel.try_into().unwrap())));
    } ));

    c.bench_function("expand_gamma", |b| b.iter(|| {
        black_box(pixels.clone().iter_mut().for_each(|n| *n = expand_gamma(*n)));
    } ));

    c.bench_function("correct_gamma", |b| b.iter(|| {
        black_box(pixels.clone().iter_mut().for_each(|n| *n = correct_gamma(*n)));
    } ));

    c.bench_function("full_to", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::SRGB, Space::LCH, pixel.try_into().unwrap())));
    } ));

    c.bench_function("full_from", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::LCH, Space::SRGB, pixel.try_into().unwrap())));
    } ));

}

criterion_group!(benches, conversions);
criterion_main!(benches);
