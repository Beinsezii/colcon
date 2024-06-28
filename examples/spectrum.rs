use colcon::{convert_space_chunked, srgb_to_irgb, Space};

const WIDTH: usize = 360;
const HEIGHT: usize = 100;

fn write_ppm(pixels: &[[f32; 3]], name: &str) {
    std::fs::write(
        format!("{}.ppm", name),
        pixels
            .iter()
            .map(|pixel| srgb_to_irgb(*pixel))
            .fold(format!("P3 {} {} 255", WIDTH, HEIGHT), |acc, it| {
                acc + "\n" + &it.map(|c| c.to_string()).join(" ")
            })
            + "\n", // newline needed for some libs
    )
    .unwrap()
}

fn main() {
    for (space, filename) in [
        (Space::HSV, "hsv"),
        (Space::CIELCH, "cie_lab"),
        (Space::OKLCH, "oklab"),
        (Space::JZCZHZ, "jzazbz"),
    ] {
        let mut data: Vec<[f32; 3]> = (0..HEIGHT)
            .map(|h| {
                (0..WIDTH)
                    .map(|w| {
                        [
                            match space {
                                Space::OKLCH => 0.70,
                                Space::JZCZHZ => 0.50,
                                _ => 0.65,
                            },
                            1.0 - h as f32 / 100.0,
                            w as f32,
                        ]
                    })
                    .collect::<Vec<[f32; 3]>>()
            })
            .flatten()
            .collect();

        data.iter_mut().for_each(|p| {
            if space == Space::HSV {
                *p = [p[2] / 360.0, p[1], p[0]]
            } else {
                p[0] *= space.srgb_quants()[95][0];
                p[1] *= space.srgb_quants()[100][1];
            }
        });

        convert_space_chunked(space, Space::SRGB, &mut data);

        write_ppm(&data, filename)
    }
}
