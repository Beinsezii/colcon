#![warn(missing_docs)]

//! Simple colorspace conversions in pure Rust.
//!
//! All conversions are in-place, except when converting to/from integer and hexadecimal.
//! Formulas are generally taken from Wikipedia with correctness verified against both
//! <https://www.easyrgb.com> and BABL.
//!
//! Currently, everything seems to check out except CIE XYZ.
//! The Wikipedia formula matches EasyRGB, but BABL uses something different.
//!
//! This crate references Standard Illuminant D65
//! when converting to/from the CIE colorspace.
//! The feature flag `D50` changes this to Illuminant D50,
//! used by BABL (GIMP) and possibly other programs.

const LAB_DELTA: f32 = 6.0 / 29.0;

const D65_X: f32 = 0.950489;
const D65_Y: f32 = 1.000000;
const D65_Z: f32 = 1.088840;

/// 'Standard' Illuminant D65.
pub const D65: [f32; 3] = [D65_X, D65_Y, D65_Z];

const D50_X: f32 = 0.964212;
const D50_Y: f32 = 1.000000;
const D50_Z: f32 = 0.825188;

/// Illuminant D50, aka "printing" illuminant.
/// Used by BABL/GIMP + others over D65, not sure why.
pub const D50: [f32; 3] = [D50_X, D50_Y, D50_Z];

#[cfg(feature = "D50")]
const ILLUMINANT: [f32; 3] = D50;

#[cfg(not(feature = "D50"))]
const ILLUMINANT: [f32; 3] = D65;

/// Defines colorspace pixels will take.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Space {
    /// Gamma-corrected sRGB.
    SRGB,
    /// Hue Saturation Value. Legacy format, LCH is preferred.
    HSV,
    /// "Linear Light RGB", aka no perceptual gamma.
    LRGB,
    /// CIE XYZ.
    /// Currently using the wikipedia formula. BABL uses a different one.
    /// Not sure which is most correct...
    XYZ,
    /// CIE LAB (Lightness a/b).
    /// Calculated in illuminant D65 unless the `D50` features flag is passed.
    LAB,
    /// CIE LCh (Lightness Chroma Hue).
    /// Cylindrical version of CIE LAB.
    LCH,
}

impl ToString for Space {
    fn to_string(&self) -> String {
        match self {
            Space::SRGB => String::from("rgba"),
            Space::HSV => String::from("hsva"),
            Space::LRGB => String::from("rgba"),
            Space::XYZ => String::from("xyza"),
            Space::LAB => String::from("laba"),
            Space::LCH => String::from("lcha"),
        }
    }
}

impl TryFrom<&str> for Space {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, ()> {
        match value.to_ascii_lowercase().trim() {
            "srgb" | "srgba" => Ok(Space::SRGB),
            "hsv" | "hsva" => Ok(Space::HSV),
            "lrgb" | "lrgba" | "rgb" | "rgba" => Ok(Space::LRGB),
            "xyz" | "xyza" => Ok(Space::XYZ),
            "lab" | "laba" => Ok(Space::LAB),
            "lch" | "lcha" => Ok(Space::LCH),
            _ => Err(()),
        }
    }
}

/// Expand gamma of a single value to linear light
#[inline]
pub fn expand_gamma(n: f32) -> f32 {
    if n <= 0.04045 {
        n / 12.92
    } else {
        ((n + 0.055) / 1.055_f32).powf(2.4)
    }
}

/// Gamma corrects a single linear light value
#[inline]
pub fn correct_gamma(n: f32) -> f32 {
    if n <= 0.0031308 {
        n * 12.92
    } else {
        1.055 * (n.powf(1.0 / 2.4)) - 0.055
    }
}

#[rustfmt::skip]
/// Runs conversion functions to convert `pixel` from one `Space` to another
/// in the least possible moves.
pub fn convert_space(from: Space, to: Space, pixel: &mut [f32; 3]) {
    match (from, to) {
        // No-op
        (Space::SRGB, Space::SRGB)
        | (Space::HSV, Space::HSV)
        | (Space::LRGB, Space::LRGB)
        | (Space::XYZ, Space::XYZ)
        | (Space::LAB, Space::LAB)
        | (Space::LCH, Space::LCH) => (),
        // Up
        (Space::SRGB, Space::HSV) => srgb_to_hsv(pixel),
        (Space::SRGB, Space::LRGB) => srgb_to_lrgb(pixel),
        (Space::SRGB, Space::XYZ) => {srgb_to_lrgb(pixel); lrgb_to_xyz(pixel)},
        (Space::SRGB, Space::LAB) => {srgb_to_lrgb(pixel); lrgb_to_xyz(pixel); xyz_to_lab(pixel)},
        (Space::SRGB, Space::LCH) => {srgb_to_lrgb(pixel); lrgb_to_xyz(pixel); xyz_to_lab(pixel); lab_to_lch(pixel)},
        (Space::LRGB, Space::XYZ) => lrgb_to_xyz(pixel),
        (Space::LRGB, Space::LAB) => {lrgb_to_xyz(pixel); xyz_to_lab(pixel)},
        (Space::LRGB, Space::LCH) => {lrgb_to_xyz(pixel); xyz_to_lab(pixel); lab_to_lch(pixel)},
        (Space::XYZ, Space::LAB) => xyz_to_lab(pixel),
        (Space::XYZ, Space::LCH) => {xyz_to_lab(pixel); lab_to_lch(pixel)},
        (Space::LAB, Space::LCH) => lab_to_lch(pixel),
        (Space::HSV, Space::LRGB) => {hsv_to_srgb(pixel); srgb_to_lrgb(pixel)},
        (Space::HSV, Space::XYZ) => {hsv_to_srgb(pixel); srgb_to_lrgb(pixel); lrgb_to_xyz(pixel)},
        (Space::HSV, Space::LAB) => {hsv_to_srgb(pixel); srgb_to_lrgb(pixel); lrgb_to_xyz(pixel); xyz_to_lab(pixel)},
        (Space::HSV, Space::LCH) => {hsv_to_srgb(pixel); srgb_to_lrgb(pixel); lrgb_to_xyz(pixel); xyz_to_lab(pixel); lab_to_lch(pixel)},
        // Down
        (Space::LCH, Space::LAB) => lch_to_lab(pixel),
        (Space::LCH, Space::XYZ) => {lch_to_lab(pixel); lab_to_xyz(pixel)},
        (Space::LCH, Space::LRGB) => {lch_to_lab(pixel); lab_to_xyz(pixel); xyz_to_lrgb(pixel)},
        (Space::LCH, Space::SRGB) => {lch_to_lab(pixel); lab_to_xyz(pixel); xyz_to_lrgb(pixel); lrgb_to_srgb(pixel)},
        (Space::LCH, Space::HSV) => {lch_to_lab(pixel); lab_to_xyz(pixel); xyz_to_lrgb(pixel); lrgb_to_srgb(pixel); srgb_to_hsv(pixel)},
        (Space::LAB, Space::XYZ) => lab_to_xyz(pixel),
        (Space::LAB, Space::LRGB) => {lab_to_xyz(pixel); xyz_to_lrgb(pixel)},
        (Space::LAB, Space::SRGB) => {lab_to_xyz(pixel); xyz_to_lrgb(pixel); lrgb_to_srgb(pixel)},
        (Space::LAB, Space::HSV) => {lab_to_xyz(pixel); xyz_to_lrgb(pixel); lrgb_to_srgb(pixel); srgb_to_hsv(pixel)},
        (Space::XYZ, Space::SRGB) => {xyz_to_lrgb(pixel); lrgb_to_srgb(pixel)},
        (Space::XYZ, Space::LRGB) => xyz_to_lrgb(pixel),
        (Space::XYZ, Space::HSV) => {xyz_to_lrgb(pixel); lrgb_to_srgb(pixel); srgb_to_hsv(pixel)},
        (Space::LRGB, Space::SRGB) => lrgb_to_srgb(pixel),
        (Space::LRGB, Space::HSV) => {lrgb_to_srgb(pixel); srgb_to_hsv(pixel)},
        (Space::HSV, Space::SRGB) => hsv_to_srgb(pixel),
    }
}

/// Same as `convert_space`, ignores the 4th value in `pixel`.
/// Just a convenience function.
pub fn convert_space_alpha(from: Space, to: Space, pixel: &mut [f32; 4]) {
    unsafe {
        convert_space(
            from,
            to,
            pixel.get_unchecked_mut(0..3).try_into().unwrap_unchecked(),
        )
    }
}

// UP {{{

/// Convert floating (0.0..1.0) RGB to integer (0..255) RGB.
/// Simply clips values > 1.0 && < 0.0.
/// Not RGB specific, but other formats typically aren't represented as integers.
pub fn srgb_to_irgb(pixel: [f32; 3]) -> [u8; 3] {
    [
        ((pixel[0] * 255.0).max(0.0).min(255.0) as u8),
        ((pixel[1] * 255.0).max(0.0).min(255.0) as u8),
        ((pixel[2] * 255.0).max(0.0).min(255.0) as u8),
    ]
}

/// Create a hexadecimal string from integer RGB.
/// Not RGB specific, but other formats typically aren't represented as hexadecimal.
pub fn irgb_to_hex(pixel: [u8; 3]) -> String {
    let mut hex = String::from("#");

    pixel.into_iter().for_each(|c| {
        [c / 16, c % 16]
            .into_iter()
            .for_each(|n| hex.push(if n >= 10 { n + 55 } else { n + 48 } as char))
    });

    hex
}

/// Convert from sRGB to HSV.
pub fn srgb_to_hsv(pixel: &mut [f32; 3]) {
    let vmin = pixel[0].min(pixel[1]).min(pixel[2]);
    let vmax = pixel[0].max(pixel[1]).max(pixel[2]);
    let dmax = vmax - vmin;

    let v = vmax;

    let (h, s) = if dmax == 0.0 {
        (0.0, 0.0)
    } else {
        let s = dmax / vmax;

        let dr = (((vmax - pixel[0]) / 6.0) + (dmax / 2.0)) / dmax;
        let dg = (((vmax - pixel[1]) / 6.0) + (dmax / 2.0)) / dmax;
        let db = (((vmax - pixel[2]) / 6.0) + (dmax / 2.0)) / dmax;

        let h = if pixel[0] == vmax {
            db - dg
        } else if pixel[1] == vmax {
            (1.0 / 3.0) + dr - db
        } else {
            (2.0 / 3.0) + dg - dr
        }
        .rem_euclid(1.0);
        (h, s)
    };
    *pixel = [h, s, v];
}

/// Convert from sRGB to Linear Light RGB.
/// <https://en.wikipedia.org/wiki/SRGB#From_sRGB_to_CIE_XYZ>
pub fn srgb_to_lrgb(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| *c = expand_gamma(*c));
}

/// Convert from Linear Light RGB to CIE XYZ.
/// <https://en.wikipedia.org/wiki/SRGB#From_sRGB_to_CIE_XYZ>
pub fn lrgb_to_xyz(pixel: &mut [f32; 3]) {
    *pixel = [
        (0.4124 * pixel[0] + 0.3576 * pixel[1] + 0.1805 * pixel[2]), // X
        (0.2126 * pixel[0] + 0.7152 * pixel[1] + 0.0722 * pixel[2]), // Y
        (0.0193 * pixel[0] + 0.1192 * pixel[1] + 0.9505 * pixel[2]), // Z
    ]
}

/// Convert from CIE XYZ to CIE LAB.
/// <https://en.wikipedia.org/wiki/CIELAB_color_space#From_CIEXYZ_to_CIELAB>
pub fn xyz_to_lab(pixel: &mut [f32; 3]) {
    pixel.iter_mut().zip(ILLUMINANT).for_each(|(c, d)| *c /= d);

    pixel.iter_mut().for_each(|c| {
        if *c > LAB_DELTA.powi(3) {
            *c = c.cbrt()
        } else {
            *c = *c / (3.0 * LAB_DELTA.powi(2)) + (4f32 / 29f32)
        }
    });

    *pixel = [
        (116.0 * pixel[1]) - 16.0,
        500.0 * (pixel[0] - pixel[1]),
        200.0 * (pixel[1] - pixel[2]),
    ]
}

/// Convert from CIE LAB to CIE LCH.
/// <https://en.wikipedia.org/wiki/CIELAB_color_space#Cylindrical_model>
pub fn lab_to_lch(pixel: &mut [f32; 3]) {
    *pixel = [
        pixel[0],
        (pixel[1].powi(2) + pixel[2].powi(2)).sqrt(),
        pixel[2].atan2(pixel[1]).to_degrees().rem_euclid(360.0),
    ];
}

// UP }}}

// DOWN {{{

/// Convert integer (0..255) RGB to floating (0.0..1.0) RGB.
/// Not RGB specific, but other formats typically aren't represented as integers.
pub fn irgb_to_srgb(pixel: [u8; 3]) -> [f32; 3] {
    [
        pixel[0] as f32 / 255.0,
        pixel[1] as f32 / 255.0,
        pixel[2] as f32 / 255.0,
    ]
}

/// Create integer RGB set from hex string.
/// Not RGB specific, but other formats typically aren't represented as hexadecimal.
pub fn hex_to_irgb(hex: &str) -> Result<[u8; 3], String> {
    let hex = hex.trim().to_ascii_uppercase();

    let mut chars = hex.chars();
    if chars.as_str().starts_with('#') {
        chars.next();
    }

    let ids: Vec<u32> = if chars.as_str().len() == 6 {
        chars
            .map(|c| {
                let u = c as u32;
                if 57 >= u && u >= 48 {
                    Ok(u - 48)
                } else if 70 >= u && u >= 65 {
                    Ok(u - 55)
                } else {
                    Err(String::from("Hex character ") + &String::from(c) + " out of bounds")
                }
            })
            .collect()
    } else {
        Err(String::from("Incorrect hex length!"))
    }?;

    Ok([
        ((ids[0]) * 16 + ids[1]) as u8,
        ((ids[2]) * 16 + ids[3]) as u8,
        ((ids[4]) * 16 + ids[5]) as u8,
    ])
}

/// Convert from HSV to sRGB.
pub fn hsv_to_srgb(pixel: &mut [f32; 3]) {
    if pixel[1] == 0.0 {
        *pixel = [pixel[2]; 3];
    } else {
        let mut var_h = pixel[0] * 6.0;
        if var_h == 6.0 {
            var_h = 0.0
        }
        let var_i = var_h.trunc();
        let var_1 = pixel[2] * (1.0 - pixel[1]);
        let var_2 = pixel[2] * (1.0 - pixel[1] * (var_h - var_i));
        let var_3 = pixel[2] * (1.0 - pixel[1] * (1.0 - (var_h - var_i)));

        *pixel = if var_i == 0.0 {
            [pixel[2], var_3, var_1]
        } else if var_i == 1.0 {
            [var_2, pixel[2], var_1]
        } else if var_i == 2.0 {
            [var_1, pixel[2], var_3]
        } else if var_i == 3.0 {
            [var_1, var_2, pixel[2]]
        } else if var_i == 4.0 {
            [var_3, var_1, pixel[2]]
        } else {
            [pixel[2], var_1, var_2]
        }
    }
}

/// Convert from Linear Light RGB to sRGB.
/// <https://en.wikipedia.org/wiki/SRGB#From_CIE_XYZ_to_sRGB>
pub fn lrgb_to_srgb(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| *c = correct_gamma(*c) );
}

/// Convert from CIE XYZ to Linear Light RGB.
/// <https://en.wikipedia.org/wiki/SRGB#From_CIE_XYZ_to_sRGB>
pub fn xyz_to_lrgb(pixel: &mut [f32; 3]) {
    *pixel = [
        3.2406 * pixel[0] - 1.5372 * pixel[1] - 0.4986 * pixel[2],
        -0.9689 * pixel[0] + 1.8758 * pixel[1] + 0.0415 * pixel[2],
        0.0557 * pixel[0] - 0.2040 * pixel[1] + 1.0570 * pixel[2],
    ];
}

/// Convert from CIE LAB to CIE XYZ.
/// <https://en.wikipedia.org/wiki/CIELAB_color_space#From_CIELAB_to_CIEXYZ>
pub fn lab_to_xyz(pixel: &mut [f32; 3]) {
    *pixel = [
        (pixel[0] + 16.0) / 116.0 + pixel[1] / 500.0,
        (pixel[0] + 16.0) / 116.0,
        (pixel[0] + 16.0) / 116.0 - pixel[2] / 200.0,
    ];

    pixel.iter_mut().for_each(|c| {
        if *c > LAB_DELTA {
            *c = c.powi(3)
        } else {
            *c = 3.0 * LAB_DELTA.powi(2) * (*c - 4f32 / 29f32)
        }
    });

    pixel.iter_mut().zip(ILLUMINANT).for_each(|(c, d)| *c *= d);
}

/// Convert from CIE LCH to CIE LAB.
/// <https://en.wikipedia.org/wiki/CIELAB_color_space#Cylindrical_model>
pub fn lch_to_lab(pixel: &mut [f32; 3]) {
    *pixel = [
        pixel[0],
        pixel[1] * pixel[2].to_radians().cos(),
        pixel[1] * pixel[2].to_radians().sin(),
    ]
}

// DOWN }}}

// TESTS {{{
#[cfg(test)]
mod tests {
    use super::*;

    const HEX: &str = "#3359F2";
    const IRGB: [u8; 3] = [51, 89, 242];
    const SRGB: [f32; 3] = [0.200000, 0.350000, 0.950000];
    const LRGB: [f32; 3] = [0.033105, 0.100482, 0.890006];

    // For both BABL and EasyRGB the decimals are off at the thousandths.
    // BABL at least works in f32 im pretty sure, though I also think it transitions by
    // reducing to LRGB as a common denominator before working back up, so maybe the rounding
    // errors happen in there as colcon will go XYZ -> LAB directly

    // taken from <https://easyrgb.com>
    // Runs @ D65
    // Their XYZ is normalized 0-100 while BABL and I believe wikipedia
    // is 0-1 so I've adjusted the decimals.

    const HSV: [f32; 3] = [0.6333, 0.7894, 0.9500];
    const XYZ: [f32; 3] = [0.21017, 0.14314, 0.85839];
    const LAB: [f32; 3] = [44.679, 40.806, -80.139];
    const LCH: [f32; 3] = [44.679, 89.930, 296.985];

    // Taken from BABL, which I honestly trust more
    // Runs @ D50 so needs to be tested with that
    // TODO: their XYZ implementation is different wikipedia/easyrgb...
    // I should probably at least add a feature flag to change it
    // <https://gitlab.gnome.org/GNOME/babl/-/blob/master/babl/babl-space.c>
    // const HSV: [f32; 3] = [0.633333, 0.789474, 0.950000];

    // const XYZ: [f32; 3] = [0.180448, 0.133343, 0.645614];
    // const LAB: [f32; 3] = [43.262680, 30.556679, -82.134712];
    // const LCH: [f32; 3] = [43.262680, 87.634590, 290.406769];

    fn pixcmp(a: [f32; 3], b: [f32; 3]) {
        assert_eq!(
            format!("{:.3} {:.3} {:.3}", a[0], a[1], a[2]),
            format!("{:.3} {:.3} {:.3}", b[0], b[1], b[2])
        );
    }

    #[test]
    fn hsv_to() {
        let mut pixel = SRGB;
        srgb_to_hsv(&mut pixel);
        pixcmp(pixel, HSV);
    }

    #[test]
    fn hsv_from() {
        let mut pixel = HSV;
        hsv_to_srgb(&mut pixel);
        pixcmp(pixel, SRGB);
    }

    #[test]
    fn lrgb_to() {
        let mut pixel = SRGB;
        srgb_to_lrgb(&mut pixel);
        pixcmp(pixel, LRGB);
    }

    #[test]
    fn lrgb_from() {
        let mut pixel = LRGB;
        lrgb_to_srgb(&mut pixel);
        pixcmp(pixel, SRGB);
    }

    #[test]
    fn xyz_to() {
        let mut pixel = LRGB;
        lrgb_to_xyz(&mut pixel);
        pixcmp(pixel, XYZ);
    }

    #[test]
    fn xyz_from() {
        let mut pixel = XYZ;
        xyz_to_lrgb(&mut pixel);
        pixcmp(pixel, LRGB);
    }

    #[test]
    fn lab_to() {
        let mut pixel = XYZ;
        xyz_to_lab(&mut pixel);
        pixcmp(pixel, LAB);
    }

    #[test]
    fn lab_from() {
        let mut pixel = LAB;
        lab_to_xyz(&mut pixel);
        pixcmp(pixel, XYZ);
    }

    #[test]
    fn lch_to() {
        let mut pixel = LAB;
        lab_to_lch(&mut pixel);
        pixcmp(pixel, LCH);
    }

    #[test]
    fn lch_from() {
        let mut pixel = LCH;
        lch_to_lab(&mut pixel);
        pixcmp(pixel, LAB);
    }

    #[test]
    fn full_to() {
        let mut pixel = SRGB;
        convert_space(Space::SRGB, Space::LCH, &mut pixel);
        pixcmp(pixel, LCH);
    }

    #[test]
    fn full_from() {
        let mut pixel = LCH;
        convert_space(Space::LCH, Space::SRGB, &mut pixel);
        pixcmp(pixel, SRGB);
    }

    #[test]
    fn sweep() {
        let mut pixel = SRGB;
        convert_space(Space::SRGB, Space::LCH, &mut pixel);
        convert_space(Space::LCH, Space::SRGB, &mut pixel);
        pixcmp(pixel, SRGB)
    }

    #[test]
    fn irgb_to() {
        assert_eq!(IRGB, srgb_to_irgb(SRGB))
    }

    #[test]
    fn irgb_from() {
        let mut srgb = irgb_to_srgb(IRGB);
        srgb.iter_mut()
            .for_each(|c| *c = (*c * 100.0).round() / 100.0);
        assert_eq!(SRGB, srgb)
    }

    #[test]
    fn hex_to() {
        assert_eq!(HEX, irgb_to_hex(IRGB))
    }

    #[test]
    fn hex_from() {
        assert_eq!(IRGB, hex_to_irgb(HEX).unwrap())
    }

    #[test]
    fn hue_wrap() {
        let it = (-1000..=2000).step_by(50);
        for a in it.clone() {
            for b in it.clone() {
                for c in it.clone() {
                    let (a, b, c) = (a as f32 / 10.0, b as f32 / 10.0, c as f32 / 10.0);
                    // lch
                    let mut pixel = [a as f32, b as f32, c as f32];
                    convert_space(Space::SRGB, Space::LCH, &mut pixel);
                    assert!(pixel[2] <= 360.0, "lch H was {}", pixel[2]);
                    assert!(pixel[2] >= 0.0, "lch H was {}", pixel[2]);
                    // hsv
                    let mut pixel = [a as f32, b as f32, c as f32];
                    convert_space(Space::SRGB, Space::HSV, &mut pixel);
                    assert!(pixel[0] <= 1.0, "hsv H was {}", pixel[0]);
                    assert!(pixel[0] >= 0.0, "hsv H was {}", pixel[0]);
                }
            }
        }
    }
}
// TESTS }}}
