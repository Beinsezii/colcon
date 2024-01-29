use colcon::{convert_space_chunked, Space};

fn main() {
    const QUANTILES: [usize; 8] = [0, 1, 5, 10, 90, 95, 99, 100];
    const STEPS: usize = 100;
    let stepsf = STEPS as f32;

    let srgb = (0..=STEPS)
        .map(move |a| {
            (0..=STEPS)
                .map(move |b| {
                    (0..=STEPS)
                        .map(move |c| [a as f32 / stepsf, b as f32 / stepsf, c as f32 / stepsf])
                        .collect::<Vec<[f32; 3]>>()
                })
                .collect::<Vec<Vec<[f32; 3]>>>()
        })
        .collect::<Vec<Vec<Vec<[f32; 3]>>>>()
        .into_iter()
        .flatten()
        .flatten()
        .collect::<Vec<[f32; 3]>>()
        .into_boxed_slice();

    assert_eq!(srgb.len() / 100 * 100, srgb.len() - 1);

    let mut results = Vec::new();

    for space in Space::ALL.iter() {
        let mut quantiles = [[123456789.0; 3]; QUANTILES.len()];
        let mut colors = srgb.clone();
        convert_space_chunked(Space::SRGB, *space, &mut colors);

        for c in 0..3 {
            let mut channel: Vec<f32> = colors.into_iter().map(|p| p[c]).collect();
            // just unwrap since SDR shouldn't nan
            channel.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

            for (qn, q) in QUANTILES.iter().enumerate() {
                quantiles[qn][c] = channel[channel.len() / 100 * q]
            }
        }

        // disable hue and enforce 0 chroma floor
        if Space::UCS_POLAR.contains(space) {
            quantiles.iter_mut().for_each(|q| q[2] = f32::INFINITY);
            quantiles[0][1] = 0.0;
        } else if space == &Space::HSV {
            quantiles.iter_mut().for_each(|q| q[0] = f32::INFINITY)
        }

        // enforce 0 lightness floor
        if Space::UCS.contains(space) || Space::UCS_POLAR.contains(space) {
            quantiles[0][0] = 0.0;
        }

        results.push(quantiles);
    }

    let mut formatted = String::new();

    for (qn, q) in QUANTILES.iter().enumerate() {
        formatted += &format!("
/// Retrieves the {} quantile for mapping a given Space back to SRGB.
/// This is useful for things like creating adjustable values in Space
/// that represent most of the SRGB range without clipping.
/// Wrapping Hue values are set to f32::INFINITY
pub const fn srgb_quant{}(&self) -> [f32; 3] {{
    match self {{",
            q, q
        );
        for (n, space) in Space::ALL.iter().enumerate() {
            formatted += &format!("
        &Space::{:?} => {:?},",
                space, results[n][qn]
            )
            .replace("inf", "f32::INFINITY");
        }
        formatted += "
    }
}"
    }

    println!("{}", formatted);
}
