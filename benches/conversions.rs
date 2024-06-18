use colcon::Space;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn pixels<const N: usize>() -> Vec<f32> {
    let size = 512;
    let mut result = Vec::<f32>::with_capacity(size * size * 3);
    for x in 1..=size {
        for y in 1..=size {
            let n = (x as f32 / size as f32 / 2.0) + (y as f32 / size as f32 / 2.0);
            result.extend_from_slice(&[n; N]);
        }
    }
    result
}

macro_rules! bench_three_generic {
    ($c: expr, $pc: expr, $f: path, $id:literal, $n:literal, $t:ty, $ts:literal) => {
        $c.bench_function(concat!($id, "_", $n, $ts), |b| {
            b.iter(|| {
                let mut pixels = $pc.clone();
                black_box(pixels.iter_mut().for_each(|pixel| $f(pixel)));
            })
        })
    };
}

macro_rules! bench_one_generic {
    ($c: expr, $ps: expr, $f: path, $id:literal, $t:ty, $ts:literal) => {
        $c.bench_function(concat!($id, "_", $ts), |b| {
            b.iter(|| {
                let mut pixels = $ps.clone();
                black_box(pixels.iter_mut().for_each(|n| *n = $f(*n)));
            })
        })
    };
}

macro_rules! bench_convert_generic {
    ($c: expr, $ps: expr, $pc: expr, $from: expr, $to:expr, $id:literal, $n:literal, $t:ty, $ts:literal) => {
        $c.bench_function(concat!($id, "_", $n, $ts), |b| {
            b.iter(|| {
                let mut pixels = $pc.clone();
                black_box(
                    pixels
                        .iter_mut()
                        .for_each(|pixel| colcon::convert_space($from, $to, pixel)),
                );
            })
        });

        $c.bench_function(concat!($id, "_", $n, $ts, "_chunk"), |b| {
            b.iter(|| {
                let mut pixels = $pc.clone();
                black_box(colcon::convert_space_chunked($from, $to, &mut pixels));
            })
        });

        $c.bench_function(concat!($id, "_", $n, $ts, "_slice"), |b| {
            b.iter(|| {
                let mut pixels = $ps.clone();
                black_box(colcon::convert_space_sliced::<_, 3>($from, $to, &mut pixels));
            })
        });
    };
}

pub fn conversions(c: &mut Criterion) {
    let pix_slice_3f32: Box<[f32]> = pixels::<3>().into_boxed_slice();
    let pix_slice_4f32: Box<[f32]> = pixels::<4>().into_boxed_slice();

    let pix_slice_3f64: Box<[f64]> = pixels::<3>()
        .into_iter()
        .map(|c| c.into())
        .collect::<Vec<f64>>()
        .into_boxed_slice();

    let pix_slice_4f64: Box<[f64]> = pixels::<4>()
        .into_iter()
        .map(|c| c.into())
        .collect::<Vec<f64>>()
        .into_boxed_slice();

    let pix_chunk_3f32: Box<[[f32; 3]]> = pixels::<3>()
        .chunks_exact(3)
        .map(|c| c.try_into().unwrap())
        .collect::<Vec<[f32; 3]>>()
        .into_boxed_slice();

    let pix_chunk_4f32: Box<[[f32; 4]]> = pixels::<4>()
        .chunks_exact(4)
        .map(|c| c.try_into().unwrap())
        .collect::<Vec<[f32; 4]>>()
        .into_boxed_slice();

    let pix_chunk_3f64: Box<[[f64; 3]]> = pixels::<3>()
        .chunks_exact(3)
        .map(|c| TryInto::<[f32; 3]>::try_into(c).unwrap().map(|n| n.into()))
        .collect::<Vec<[f64; 3]>>()
        .into_boxed_slice();

    let pix_chunk_4f64: Box<[[f64; 4]]> = pixels::<4>()
        .chunks_exact(4)
        .map(|c| TryInto::<[f32; 4]>::try_into(c).unwrap().map(|n| n.into()))
        .collect::<Vec<[f64; 4]>>()
        .into_boxed_slice();

    macro_rules! bench_three {
        ($f: path, $id:literal) => {
            bench_three_generic!(c, pix_chunk_3f32, $f, $id, 3, f32, "f32");
            bench_three_generic!(c, pix_chunk_3f64, $f, $id, 3, f64, "f64");
            bench_three_generic!(c, pix_chunk_4f32, $f, $id, 4, f32, "f32");
            bench_three_generic!(c, pix_chunk_4f64, $f, $id, 4, f64, "f64");
        };
    }

    macro_rules! bench_one {
        ($f: path, $id:literal) => {
            bench_one_generic!(c, pix_slice_3f32, $f, $id, f32, "f32");
            bench_one_generic!(c, pix_slice_3f32, $f, $id, f64, "f64");
        };
    }

    macro_rules! bench_convert {
        ($from: expr, $to:expr, $id:literal) => {
            bench_convert_generic!(c, pix_slice_3f32, pix_chunk_3f32, $from, $to, $id, 3, f32, "f32");
            bench_convert_generic!(c, pix_slice_3f64, pix_chunk_3f64, $from, $to, $id, 3, f64, "f64");
            bench_convert_generic!(c, pix_slice_4f32, pix_chunk_4f32, $from, $to, $id, 4, f32, "f32");
            bench_convert_generic!(c, pix_slice_4f64, pix_chunk_4f64, $from, $to, $id, 4, f64, "f64");
        };
    }

    // Forward
    bench_three!(colcon::srgb_to_lrgb, "srgb_to_lrgb");
    bench_three!(colcon::lrgb_to_xyz, "lrgb_to_xyz");
    bench_three!(colcon::xyz_to_cielab, "xyz_to_cielab");
    bench_three!(colcon::xyz_to_oklab, "xyz_to_oklab");
    bench_three!(colcon::xyz_to_jzazbz, "xyz_to_jzazbz");
    bench_three!(colcon::lab_to_lch, "lab_to_lch");
    bench_three!(colcon::srgb_to_hsv, "srgb_to_hsv");
    // Backward
    bench_three!(colcon::lch_to_lab, "lch_to_lab");
    bench_three!(colcon::jzazbz_to_xyz, "jzazbz_to_xyz");
    bench_three!(colcon::oklab_to_xyz, "oklab_to_xyz");
    bench_three!(colcon::cielab_to_xyz, "cielab_to_xyz");
    bench_three!(colcon::xyz_to_lrgb, "xyz_to_lrgb");
    bench_three!(colcon::lrgb_to_srgb, "lrgb_to_srgb");
    bench_three!(colcon::hsv_to_srgb, "hsv_to_srgb");

    bench_one!(colcon::srgb_eotf, "srgb_eotf");
    bench_one!(colcon::srgb_oetf, "srgb_oetf");
    bench_one!(colcon::pq_eotf, "pq_eotf");
    bench_one!(colcon::pq_oetf, "pq_oetf");

    bench_convert!(Space::SRGB, Space::CIELCH, "full_forward");
    bench_convert!(Space::CIELCH, Space::SRGB, "full_backward");
    bench_convert!(Space::LRGB, Space::XYZ, "minimal");
}

criterion_group!(benches, conversions);
criterion_main!(benches);
