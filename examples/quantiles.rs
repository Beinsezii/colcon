use colcon::{convert_space_sliced, unweave, Space};

fn main() {
    const STEPS: usize = 100;
    let stepsf = STEPS as f64;

    let srgb = (0..=STEPS)
        .map(move |a| {
            (0..=STEPS)
                .map(move |b| {
                    (0..=STEPS)
                        .map(move |c| [a as f64 / stepsf, b as f64 / stepsf, c as f64 / stepsf])
                        .collect::<Vec<[f64; 3]>>()
                })
                .collect::<Vec<Vec<[f64; 3]>>>()
        })
        .collect::<Vec<Vec<Vec<[f64; 3]>>>>()
        .into_iter()
        .flatten()
        .flatten()
        .flatten()
        .collect::<Vec<f64>>()
        .into_boxed_slice();

    assert_eq!(srgb.len() % 3, 0);
    assert_eq!(((srgb.len() / 3) - 1) % 100, 0);

    let mut formatted = String::from(
        "pub const fn srgb_quants(space: &crate::Space) -> [[f32; 3]; 101] {
    match space {",
    );

    for space in Space::ALL.iter() {
        let mut quantiles = [[123456789.0; 3]; 101];
        let mut colors = srgb.clone();
        convert_space_sliced::<_, 3>(Space::SRGB, *space, &mut colors);

        for (nc, mut channel) in unweave::<_, 3>(&colors).into_iter().enumerate() {
            // just unwrap since SDR shouldn't nan
            channel.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

            for n in 0..=100 {
                quantiles[n][nc] = channel[channel.len() / 100 * n]
            }
        }

        // disable hue and enforce 0 chroma floor
        // otherwise JZCZHZ and CIELCH (C) are something like 1e-16
        if Space::UCS_POLAR.contains(space) {
            quantiles.iter_mut().for_each(|q| q[2] = f64::INFINITY);
            quantiles[0][1] = 0.0;
        } else if space == &Space::HSV {
            quantiles.iter_mut().for_each(|q| q[0] = f64::INFINITY)
        }

        // enforce 0 lightness floor.
        // otherwise JZCZHZ and CIELCH (L) are something like 1e-16
        if Space::UCS.contains(space) || Space::UCS_POLAR.contains(space) {
            quantiles[0][0] = 0.0;
        }

        formatted +=
            &format!("\n        &crate::Space::{:?} => {:?},", space, quantiles).replace("inf", "f32::INFINITY");
    }

    formatted += "
    }
}";

    println!("{}", formatted);
}
