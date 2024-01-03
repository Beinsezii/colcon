use colcon::{convert_space_chunked, srgb_to_irgb, Space};

const WIDTH: usize = 360;
const HEIGHT: usize = 100;

fn write_ppm(pixels: &[[f32; 3]], name: &str) {
        std::fs::write(
            format!("{}.ppm", name),
            pixels.iter().map(|pixel| srgb_to_irgb(*pixel))
                .fold(format!("P3 {} {} 255", WIDTH, HEIGHT), |acc, it| {
                    acc + "\n" + &it.map(|c| c.to_string()).join(" ")
                })
                + "\n", // newline needed for some libs
        )
        .unwrap()

}

fn main() {
    let mut pixels = [[[65.0, 0.0, 0.0]; WIDTH]; HEIGHT];

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            pixels[y][x][1] = 100.0 - (y as f32);
            pixels[y][x][2] = x as f32;
        }
    }

    for (space, filename) in [
        (Space::HSV, "hsv"),
        (Space::LCH, "cie_lab"),
        (Space::OKLCH, "oklab"),
        (Space::JZCZHZ, "jzazbz"),
    ] {
        let mut data: Vec<[f32; 3]> = pixels
            .iter()
            .map(|a| a.to_vec())
            .reduce(|mut acc, mut it| {
                acc.append(&mut it);
                acc
            })
            .unwrap();
        let add = match space {
            Space::JZCZHZ => [-15.0, 0.0, 0.0],
            Space::OKLCH => [5.0, 0.0, 0.0],
            _ => [0.0, 0.0, 0.0],
        };
        let div = match space {
            Space::HSV => [100.0, 100.0, 360.0],
            Space::OKLCH => [100.0, 400.0, 1.0],
            Space::JZCZHZ => [5650.0, 5650.0, 1.0],
            _ => [1.0, 1.0, 1.0],
        };
        data.iter_mut()
            .for_each(|p| {p.iter_mut()
                .zip(add.iter()).zip(div.iter())
                .for_each(|((c, a), d)| {*c += *a; *c /= *d});
                if space == Space::HSV  {*p = [p[2], p[1], p[0]]}
            });
        convert_space_chunked(space, Space::SRGB, &mut data);

        write_ppm(&data, filename)
    }

    let mut ictcp: Vec<[f32; 3]> = pixels.iter().map(|a| a.to_vec()).reduce(|mut acc, mut it| {
            acc.append(&mut it);
            acc
        }).unwrap();
    ictcp.iter_mut().for_each(|p| {
        p[0] += 15.0;
        p[0] /= 800.0;
        p[1] /= 800.0;
        p[2] += 60.0;
        colcon::lch_to_lab(p);
        colcon::_ictcp_to_lrgb(p);
        colcon::lrgb_to_srgb(p);
    });
    write_ppm(&ictcp, "ictcp");
}
