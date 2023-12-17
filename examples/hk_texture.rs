use colcon::{convert_space, hk_high2023_comp, srgb_to_irgb};

fn main() {
    const TX_WIDTH: usize = 72;
    const TX_HEIGHT: usize = 100;
    const GAP: usize = 5;
    const COUNT: usize = 2;
    const WIDTH: usize = TX_WIDTH * COUNT + GAP * (COUNT - 1);
    const HEIGHT: usize = TX_HEIGHT;

    let mut pixels = [ [ [50.0f32, 0.0, 0.0] ; WIDTH ] ; HEIGHT];

    for x in 0..TX_WIDTH {
        for y in 0..TX_HEIGHT {
            let lc = 100.0 - 50.0 / TX_HEIGHT as f32 * y as f32 * 2.0;
            let h = 360.0 / TX_WIDTH as f32 * x as f32;
            let mut pixel = [
                lc, lc, h
            ];
            pixels[y][x] = pixel;
            hk_high2023_comp(&mut pixel);
            pixels[y][x+TX_WIDTH+GAP] = pixel
        }
    }

    std::fs::write(
        "hk_texture.ppm",
        pixels
            .map(|row| {
                row.map(|pixel| {
                    let mut pixel = pixel;
                    convert_space(colcon::Space::LCH, colcon::Space::SRGB, &mut pixel);
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
