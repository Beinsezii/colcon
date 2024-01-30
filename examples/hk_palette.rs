use colcon::{convert_space, hk_high2023_comp, srgb_to_irgb};

fn main() {
    const SQUARE: usize = 3;
    const GAP: usize = 1;
    const COLS: usize = 12;
    const ROWS: usize = 2;
    const WIDTH: usize = SQUARE * COLS;
    const HEIGHT: usize = SQUARE * ROWS + GAP * (ROWS - 1);

    let mut pixels = [[[50.0f32, 0.0, 0.0]; WIDTH]; HEIGHT];

    let lch: [[f32; 3]; COLS] = (0..COLS)
        .map(|n| [65.0, 65.0, n as f32 * (360.0 / (COLS as f32))])
        .collect::<Vec<[f32; 3]>>()
        .try_into()
        .unwrap();

    let mut lch_hk_2023 = lch;
    lch_hk_2023.iter_mut().for_each(|pixel| hk_high2023_comp(pixel));

    for (ln, lch) in [lch, lch_hk_2023].iter().enumerate() {
        let i = ln * (SQUARE + GAP);
        for (n, p) in lch.iter().enumerate() {
            pixels[i..(i + SQUARE)].iter_mut().for_each(|row| {
                row.get_mut((n * SQUARE)..(n * SQUARE + SQUARE))
                    .unwrap()
                    .fill(*p)
            });
        }
    }

    std::fs::write(
        "hk_palette.ppm",
        pixels
            .map(|row| {
                row.map(|pixel| {
                    let mut pixel = pixel;
                    convert_space(colcon::Space::CIELCH, colcon::Space::SRGB, &mut pixel);
                    srgb_to_irgb(pixel)
                })
            })
            .iter()
            .fold(format!("P3 {} {} 255", WIDTH, HEIGHT), |acc, e| {
                acc + "\n" + &e.map(|p| p.map(|c| c.to_string()).join(" ")).join("\n")
            })
            + "\n", // newline needed for some libs
    )
    .unwrap()
}
