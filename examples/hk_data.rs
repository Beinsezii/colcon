use colcon::hk_high2023;

fn main() {
    println!(
        "{}",
        (0..36)
            .map(|n| {
                let pixel = [100.0, 100.0, n as f32 * 10.0];
                format!("\nHK_DELTA_2023({:?}) = {}", pixel, hk_high2023(&pixel))
            })
            .reduce(|acc, e| { acc + &e })
            .unwrap()
    );

    let samples = 360 * 100;
    println!(
        "Mean HK 2023 Delta: {}",
        (0..samples)
            .map(|n| hk_high2023(&[100.0, 100.0, (360.0 / (samples as f32) * (n as f32))]))
            .sum::<f32>()
            / samples as f32
    );
}
