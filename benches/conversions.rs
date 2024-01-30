use criterion::{black_box, criterion_group, criterion_main, Criterion};
use colcon::{Space, convert_space};

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
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::srgb_to_lrgb(pixel.try_into().unwrap())));
    } ));

    c.bench_function("lrgb_to_xyz", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::lrgb_to_xyz(pixel.try_into().unwrap())));
    } ));

    c.bench_function("xyz_to_lab", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::xyz_to_lab(pixel.try_into().unwrap())));
    } ));

    c.bench_function("xyz_to_oklab", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::xyz_to_oklab(pixel.try_into().unwrap())));
    } ));

    c.bench_function("xyz_to_jzazbz", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::xyz_to_jzazbz(pixel.try_into().unwrap())));
    } ));

    c.bench_function("lab_to_lch", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::lab_to_lch(pixel.try_into().unwrap())));
    } ));

    c.bench_function("lch_to_lab", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::lch_to_lab(pixel.try_into().unwrap())));
    } ));

    c.bench_function("jzazbz_to_xyz", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::jzazbz_to_xyz(pixel.try_into().unwrap())));
    } ));

    c.bench_function("oklab_to_xyz", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::oklab_to_xyz(pixel.try_into().unwrap())));
    } ));

    c.bench_function("lab_to_xyz", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::lab_to_xyz(pixel.try_into().unwrap())));
    } ));

    c.bench_function("xyz_to_lrgb", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::xyz_to_lrgb(pixel.try_into().unwrap())));
    } ));

    c.bench_function("lrgb_to_srgb", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::lrgb_to_srgb(pixel.try_into().unwrap())));
    } ));

    c.bench_function("srgb_to_hsv", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::srgb_to_hsv(pixel.try_into().unwrap())));
    } ));

    c.bench_function("hsv_to_srgb", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| colcon::hsv_to_srgb(pixel.try_into().unwrap())));
    } ));

    c.bench_function("srgb_eotf", |b| b.iter(|| {
        black_box(pixels.clone().iter_mut().for_each(|n| *n = colcon::srgb_eotf(*n)));
    } ));

    c.bench_function("srgb_eotf_inverse", |b| b.iter(|| {
        black_box(pixels.clone().iter_mut().for_each(|n| *n = colcon::srgb_eotf_inverse(*n)));
    } ));

    c.bench_function("full_to", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::SRGB, Space::CIELCH, pixel.try_into().unwrap())));
    } ));

    c.bench_function("full_from", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::CIELCH, Space::SRGB, pixel.try_into().unwrap())));
    } ));

    c.bench_function("full_to_chunk", |b| b.iter(|| {
        black_box(colcon::convert_space_chunked(Space::CIELCH, Space::SRGB, pixels.chunks_exact(3).map(|chunk| chunk.try_into().unwrap()).collect::<Vec<[f32; 3]>>().as_mut_slice()));
    } ));

    c.bench_function("full_from_chunk", |b| b.iter(|| {
        black_box(colcon::convert_space_chunked(Space::CIELCH, Space::SRGB, &mut pixels.chunks_exact(3).map(|chunk| chunk.try_into().unwrap()).collect::<Vec<[f32; 3]>>().as_mut_slice()));
    } ));

    c.bench_function("full_to_slice", |b| b.iter(|| {
        black_box(colcon::convert_space_sliced(Space::CIELCH, Space::SRGB, &mut pixels.clone()));
    } ));

    c.bench_function("full_from_slice", |b| b.iter(|| {
        black_box(colcon::convert_space_sliced(Space::CIELCH, Space::SRGB, &mut pixels.clone()));
    } ));

    c.bench_function("single", |b| b.iter(|| {
        black_box(pixels.clone().chunks_exact_mut(3).for_each(|pixel| convert_space(Space::LRGB, Space::XYZ, pixel.try_into().unwrap())));
    } ));

    c.bench_function("single_chunk", |b| b.iter(|| {
        black_box(colcon::convert_space_chunked(Space::LRGB, Space::XYZ, pixels.chunks_exact(3).map(|chunk| chunk.try_into().unwrap()).collect::<Vec<[f32; 3]>>().as_mut_slice()));
    } ));

    c.bench_function("single_slice", |b| b.iter(|| {
        black_box(colcon::convert_space_sliced(Space::LRGB, Space::XYZ, &mut pixels.clone()));
    } ));
}

criterion_group!(benches, conversions);
criterion_main!(benches);
