use colcon::{convert_space, srgb_to_irgb};


const K_HIGH2022: [f32; 4] = [
    0.1644,
    0.0603,
    0.1307,
    0.0060,
];

const K_HIGH2022_HALF: [f32; 2] = [
    0.1825,
    0.0909,
];

fn hk_2023_fby(h: f32) -> f32 {
    K_HIGH2022[0] * ((h - 90.0) / 2.0).to_radians().sin().abs() + K_HIGH2022[1]
}

fn hk_1991_fby(h: f32) -> f32 {
    K_HIGH2022_HALF[0] * ((h - 90.0) / 2.0).to_radians().sin().abs() + K_HIGH2022_HALF[1]
}

fn hk_2023_fr(h: f32) -> f32 {
    if h <= 90.0 || h >= 270.0 {
        K_HIGH2022[2] * h.to_radians().cos().abs() + K_HIGH2022[3]
    } else {
        0.0
    }
}

/// Calculate L offset for a pixel according to
/// https://onlinelibrary.wiley.com/doi/10.1002/col.22839
fn hk_2023(pixel: [f32; 3]) -> f32 {
    let fby = hk_2023_fby(pixel[2]);
    println!("2023 Fby({}) = {}", pixel[2], fby);
    let fr = hk_2023_fr(pixel[2]);
    (fby + fr) * pixel[1]
}

fn hk_1991(pixel: [f32; 3]) -> f32 {
    let fby = hk_1991_fby(pixel[2]);
    println!("1991 Fby({}) = {}", pixel[2], fby);
    fby * pixel[1]
}

fn apply_hk_2023(pixel: &mut [f32; 3]) {
    let hk = hk_2023(*pixel);
    // println!("HK2023 for {:?}: {:?}", pixel, hk);
    pixel[0] += 10.0 - hk;
}

fn apply_hk_1991(pixel: &mut [f32; 3]) {
    let hk = hk_1991(*pixel);
    // println!("HK1991 for {:?}: {:?}", pixel, hk);
    pixel[0] += 10.0 - hk;
}

fn main() {
    const SQUARE: usize = 3;
    const GAP: usize = 1;
    const COLS: usize = 12;
    const ROWS: usize = 3;
    const WIDTH: usize = SQUARE * COLS;
    const HEIGHT: usize = SQUARE * ROWS + GAP * (ROWS - 1);

    // 12 3x3 squares in a row vertically separated by a 1x1 line
    let mut pixels = [[ [50.0f32, 0.0, 0.0]; WIDTH]; HEIGHT];

    // break up hue over 4 offset rotations so colors arent all neighboring
    // let lch: [[f32; 3]; 12] = (0..48).step_by(4).map(|n| [65.0, 65.0, (n % 12 + n / 12) as f32 * 30.0]).collect::<Vec<[f32; 3]>>().try_into().unwrap();
    let lch: [[f32; 3]; COLS] = (0..COLS).map(|n| [65.0, 65.0, n as f32 * (360.0 / (COLS as f32))]).collect::<Vec<[f32; 3]>>().try_into().unwrap();

    let mut lch_hk_1991 = lch;
    lch_hk_1991.iter_mut().for_each(|pixel| apply_hk_1991(pixel));

    let mut lch_hk_2023 = lch;
    lch_hk_2023.iter_mut().for_each(|pixel| apply_hk_2023(pixel));

    for (ln, lch) in [lch, lch_hk_1991, lch_hk_2023].iter().enumerate() {
        let i = ln * (SQUARE + GAP);
        for (n, p) in lch.iter().enumerate() {
            pixels[i..(i+SQUARE)].iter_mut().for_each(|row| row.get_mut((n*SQUARE)..(n*SQUARE+SQUARE)).unwrap().fill(*p));
        }
    }

    std::fs::write("hk.ppm", pixels.map(|row| row.map(|pixel| {
        let mut pixel = pixel;
        convert_space(colcon::Space::LCH, colcon::Space::SRGB, &mut pixel);
        srgb_to_irgb(pixel)
    })).iter().fold(format!("P3 {} {} 255", WIDTH, HEIGHT), |acc, e| {
        acc + "\n" + &e.map(|p| p.map(|c| c.to_string()).join(" ")).join("\n")
    } ) + "\n" // newline needed for some libs
        ).unwrap()
}
