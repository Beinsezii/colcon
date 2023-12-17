#![warn(missing_docs)]

//! Simple colorspace conversions in pure Rust.
//!
//! All conversions are in-place, except when converting to/from integer and hexadecimal.
//! Formulas are generally taken from Wikipedia with correctness verified against both
//! <https://www.easyrgb.com> and BABL.
//!
//! Helmholtz-Kohlrausch compensation formulae sourced from
//! <https://onlinelibrary.wiley.com/doi/10.1002/col.22839>
//!
//! This crate references CIE Standard Illuminant D65 for functions to/from CIE XYZ

use core::cmp::Ordering;
use core::ffi::{c_char, CStr};

// ### MATRICES ### {{{
// CIE XYZ
const XYZ65_MAT: [[f32; 3]; 3] = [
    [0.4124, 0.3576, 0.1805],
    [0.2126, 0.7152, 0.0722],
    [0.0193, 0.1192, 0.9505],
];

const XYZ65_MAT_INV: [[f32; 3]; 3] = [
    [3.2406, -1.5372, -0.4986],
    [-0.9689, 1.8758, 0.0415],
    [0.0557, -0.2040, 1.0570],
];

// OKLAB
const OKLAB_M1: [[f32; 3]; 3] = [
    [0.8189330101, 0.0329845436, 0.0482003018],
    [0.3618667424, 0.9293118715, 0.2643662691],
    [-0.1288597137, 0.0361456387, 0.6338517070],
];
const OKLAB_M2: [[f32; 3]; 3] = [
    [0.2104542553, 1.9779984951, 0.0259040371],
    [0.7936177850, -2.4285922050, 0.7827717662],
    [-0.0040720468, 0.4505937099, -0.8086757660],
];
const OKLAB_M1_INV: [[f32; 3]; 3] = [
    [1.2270138511, -0.0405801784, -0.0763812845],
    [-0.5577999807, 1.1122568696, -0.4214819784],
    [0.281256149, -0.0716766787, 1.5861632204],
];
const OKLAB_M2_INV: [[f32; 3]; 3] = [
    [0.9999999985, 1.0000000089, 1.0000000547],
    [0.3963377922, -0.1055613423, -0.0894841821],
    [0.2158037581, -0.0638541748, -1.2914855379],
];
/// 3 * 3x3 Matrix multiply
fn matmul3(pixel: [f32; 3], matrix: [[f32; 3]; 3]) -> [f32; 3] {
    [
        pixel[0] * matrix[0][0] + pixel[1] * matrix[1][0] + pixel[2] * matrix[2][0],
        pixel[0] * matrix[0][1] + pixel[1] * matrix[1][1] + pixel[2] * matrix[2][1],
        pixel[0] * matrix[0][2] + pixel[1] * matrix[1][2] + pixel[2] * matrix[2][2],
    ]
}

/// Transposed 3 * 3x3 matrix multiply
fn matmul3t(pixel: [f32; 3], matrix: [[f32; 3]; 3]) -> [f32; 3] {
    [
        pixel[0] * matrix[0][0] + pixel[1] * matrix[0][1] + pixel[2] * matrix[0][2],
        pixel[0] * matrix[1][0] + pixel[1] * matrix[1][1] + pixel[2] * matrix[1][2],
        pixel[0] * matrix[2][0] + pixel[1] * matrix[2][1] + pixel[2] * matrix[2][2],
    ]
}
// ### MATRICES ### }}}

// ### UTILITIES ### {{{

/// Standard Illuminant D65.
pub const D65: [f32; 3] = [0.9504559270516716, 1.0, 1.0890577507598784];

const LAB_DELTA: f32 = 6.0 / 29.0;

/// Expand gamma of a single value to linear light
#[inline]
#[no_mangle]
pub extern "C" fn expand_gamma(n: f32) -> f32 {
    if n <= 0.04045 {
        n / 12.92
    } else {
        ((n + 0.055) / 1.055_f32).powf(2.4)
    }
}

/// Gamma corrects a single linear light value
#[inline]
#[no_mangle]
pub extern "C" fn correct_gamma(n: f32) -> f32 {
    if n <= 0.0031308 {
        n * 12.92
    } else {
        1.055 * (n.powf(1.0 / 2.4)) - 0.055
    }
}
// ### UTILITIES ### }}}

// ### Helmholtz-Kohlrausch ### {{{

/// Extended K-values from High et al 2021/2022
const K_HIGH2022: [f32; 4] = [0.1644, 0.0603, 0.1307, 0.0060];

/// Mean value of the HK delta, High et al 2023 implementation.
/// Measured with 36000 steps in the hk_exmample file @ 100 C(ab)
/// Cannot make a const fn: https://github.com/rust-lang/rust/issues/57241
pub const HIGH2023_MEAN: f32 = 20.956442;

#[inline]
fn hk_2023_fby(h: f32) -> f32 {
    K_HIGH2022[0] * ((h - 90.0) / 2.0).to_radians().sin().abs() + K_HIGH2022[1]
}

#[inline]
fn hk_2023_fr(h: f32) -> f32 {
    if h <= 90.0 || h >= 270.0 {
        K_HIGH2022[2] * h.to_radians().cos().abs() + K_HIGH2022[3]
    } else {
        0.0
    }
}

/// Returns difference in perceptual lightness based on hue, aka the Helmholtz-Kohlrausch effect.
/// High et al 2023 implementation.
#[no_mangle]
pub extern "C" fn hk_delta_2023(lch: &[f32; 3]) -> f32 {
    (hk_2023_fby(lch[2]) + hk_2023_fr(lch[2])) * lch[1]
}

/// Compensates LCH's L value for the Helmholtz-Kohlrausch effect.
/// High et al 2023 implementation.
#[no_mangle]
pub extern "C" fn hk_comp_2023(lch: &mut [f32; 3]) {
    lch[0] += HIGH2023_MEAN * (lch[1] / 100.0) - hk_delta_2023(lch)
}

// ### Helmholtz-Kohlrausch ### }}}

// ### Space ### {{{

/// Defines colorspace pixels will take.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Space {
    /// Gamma-corrected sRGB.
    SRGB,

    /// Hue Saturation Value.
    /// A UCS typically preferred for modern applications
    HSV,

    /// Linear RGB. IEC 61966-2-1:1999 transferred
    LRGB,

    /// 1931 CIE XYZ @ D65.
    XYZ,

    /// CIE L*a*b*. Lightness, red/green chromacity, yellow/blue chromacity.
    /// Old UCS with many known flaws
    LAB,

    /// CIE L*C*Hab. Lightness, Chroma, Hue
    /// Cylindrical version of CIE L*a*b*.
    LCH,

    /// Oklab <https://bottosson.github.io/posts/oklab/>
    /// Modern UCS, used in CSS Color Module Level 4
    OKLAB,

    /// Cylindrical version of OKLAB.
    OKLCH,
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
            "oklab" | "oklaba" => Ok(Space::OKLAB),
            "oklch" | "oklcha" => Ok(Space::OKLCH),
            _ => Err(()),
        }
    }
}

impl core::fmt::Display for Space {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::write(f, core::format_args!("{}", match self {
            Self::SRGB => "sRGB",
            Self::HSV => "HSV",
            Self::LRGB => "RGB",
            Self::XYZ => "CIE XYZ",
            Self::LAB => "CIE L*a*b*",
            Self::LCH => "CIE L*C*Hab",
            Self::OKLAB => "Oklab",
            Self::OKLCH => "Oklch",
        }))
    }
}

impl PartialOrd for Space {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }
        Some(match self {
            // Base
            Space::SRGB => Ordering::Less,

            // Endcaps
            Space::HSV => Ordering::Greater,
            Space::LCH => Ordering::Greater,
            Space::OKLCH => Ordering::Greater,

            // Common intermittents
            Space::LRGB => match other {
                Space::SRGB | Space::HSV => Ordering::Greater,
                _ => Ordering::Less,
            },
            Space::XYZ => match other {
                Space::SRGB | Space::LRGB | Space::HSV => Ordering::Greater,
                _ => Ordering::Less,
            },

            // LAB Branches
            Space::LAB => match other {
                Space::LCH => Ordering::Less,
                _ => Ordering::Greater,
            },
            Space::OKLAB => match other {
                Space::OKLCH => Ordering::Less,
                _ => Ordering::Greater,
            },
        })
    }
}

impl Space {
    /// Returns 3 channels letters for user-facing colorspace controls
    pub fn channels(&self) -> [char; 3] {
        match self {
            Space::SRGB => ['r', 'g', 'b'],
            Space::HSV => ['h', 's', 'v'],
            Space::LRGB => ['r', 'g', 'b'],
            Space::XYZ => ['x', 'y', 'z'],
            Space::LAB => ['l', 'a', 'b'],
            Space::LCH => ['l', 'c', 'h'],
            Space::OKLAB => ['l', 'a', 'b'],
            Space::OKLCH => ['l', 'c', 'h'],
        }
    }

    /// All color spaces
    pub const ALL: [Space; 8] = [Space::SRGB, Space::HSV, Space::LRGB, Space::XYZ, Space::LAB, Space::LCH, Space::OKLAB, Space::OKLCH];

    /// Uniform color spaces
    pub const UCS: [Space; 2] = [Space::LAB, Space::OKLAB];

    /// Uniform color spaces in cylindrical/polar format
    pub const UCS_POLAR: [Space; 2] = [Space::LCH, Space::OKLCH];

    /// RGB/Tristimulus color spaces
    pub const TRI: [Space; 3] = [Space::SRGB, Space::LRGB, Space::XYZ];
}


// ### Space ### }}}

// ### Convert Space ### {{{

/// Runs conversion functions to convert `pixel` from one `Space` to another
/// in the least possible moves.
pub fn convert_space(from: Space, to: Space, pixel: &mut [f32; 3]) {
    let mut pixels = [*pixel];
    convert_space_chunked(from, to, &mut pixels);
    *pixel = pixels[0]
}

/// Runs conversion functions to convert `pixel` from one `Space` to another
/// in the least possible moves. Caches conversion graph for faster iteration.
pub fn convert_space_chunked(from: Space, to: Space, pixels: &mut [[f32; 3]]) {
    if from > to {
        match from {
            Space::SRGB => unreachable!(),
            Space::HSV => {
                pixels.iter_mut().for_each(|pixel| hsv_to_srgb(pixel));
                convert_space_chunked(Space::SRGB, to, pixels)
            }
            Space::LRGB => {
                pixels.iter_mut().for_each(|pixel| lrgb_to_srgb(pixel));
                convert_space_chunked(Space::SRGB, to, pixels)
            }
            Space::XYZ => {
                pixels.iter_mut().for_each(|pixel| xyz_to_lrgb(pixel));
                convert_space_chunked(Space::LRGB, to, pixels)
            }
            Space::LAB => {
                pixels.iter_mut().for_each(|pixel| lab_to_xyz(pixel));
                convert_space_chunked(Space::XYZ, to, pixels)
            }
            Space::LCH => {
                pixels.iter_mut().for_each(|pixel| lch_to_lab(pixel));
                convert_space_chunked(Space::LAB, to, pixels)
            }
            Space::OKLAB => {
                pixels.iter_mut().for_each(|pixel| oklab_to_xyz(pixel));
                convert_space_chunked(Space::XYZ, to, pixels)
            }
            Space::OKLCH => {
                pixels.iter_mut().for_each(|pixel| lch_to_lab(pixel));
                convert_space_chunked(Space::OKLAB, to, pixels)
            }
        }
    } else if from < to {
        match from {
            // Endcaps
            Space::HSV => unreachable!(),
            Space::LCH => unreachable!(),
            Space::OKLCH => unreachable!(),

            Space::SRGB => match to {
                Space::HSV => pixels.iter_mut().for_each(|pixel| srgb_to_hsv(pixel)),
                _ => {
                    pixels.iter_mut().for_each(|pixel| srgb_to_lrgb(pixel));
                    convert_space_chunked(Space::LRGB, to, pixels)
                }
            },
            Space::LRGB => {
                pixels.iter_mut().for_each(|pixel| lrgb_to_xyz(pixel));
                convert_space_chunked(Space::XYZ, to, pixels)
            }
            Space::XYZ => match to {
                Space::LAB | Space::LCH => {
                    pixels.iter_mut().for_each(|pixel| xyz_to_lab(pixel));
                    convert_space_chunked(Space::LAB, to, pixels)
                }
                Space::OKLAB | Space::OKLCH => {
                    pixels.iter_mut().for_each(|pixel| xyz_to_oklab(pixel));
                    convert_space_chunked(Space::OKLAB, to, pixels)
                }
                _ => unreachable!("XYZ tried to promote to {}", to.to_string()),
            },
            Space::LAB => pixels.iter_mut().for_each(|pixel| lab_to_lch(pixel)),
            Space::OKLAB => pixels.iter_mut().for_each(|pixel| lab_to_lch(pixel)),
        }
    }
}

/// Runs conversion functions to convert `pixel` from one `Space` to another
/// in the least possible moves. Caches conversion graph for faster iteration.
/// Ignores remainder values in slice.
pub fn convert_space_sliced(from: Space, to: Space, pixels: &mut [f32]) {
    // How long has this been in unstable...
    // convert_space_chunked(from, to, pixels.as_chunks_mut::<3>().0);
    pixels
        .chunks_exact_mut(3)
        .for_each(|pixel| convert_space(from, to, pixel.try_into().unwrap()));
}

/// Same as `convert_space_sliced` but with FFI types.
/// Returns 0 on success, 1 on invalid `from`, 2 on invalid `to`, 3 on invalid `pixels`
#[no_mangle]
pub extern "C" fn convert_space_ffi(
    from: *const c_char,
    to: *const c_char,
    pixels: *mut f32,
    len: usize,
) -> i32 {
    let from = unsafe {
        if from.is_null() {
            return 1;
        } else {
            if let Some(s) = CStr::from_ptr(from)
                .to_str()
                .ok()
                .map(|s| Space::try_from(s).ok())
                .flatten()
            {
                s
            } else {
                return 1;
            }
        }
    };
    let to = unsafe {
        if to.is_null() {
            return 2;
        } else {
            if let Some(s) = CStr::from_ptr(to)
                .to_str()
                .ok()
                .map(|s| Space::try_from(s).ok())
                .flatten()
            {
                s
            } else {
                return 2;
            }
        }
    };
    let pixels = unsafe {
        if pixels.is_null() {
            return 3;
        } else {
            core::slice::from_raw_parts_mut(pixels, len)
        }
    };
    convert_space_sliced(from, to, pixels);
    0
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

// ### Convert Space ### }}}

// ### UP ###  {{{

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
#[no_mangle]
pub extern "C" fn srgb_to_hsv(pixel: &mut [f32; 3]) {
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
#[no_mangle]
pub extern "C" fn srgb_to_lrgb(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| *c = expand_gamma(*c));
}

/// Convert from Linear Light RGB to CIE XYZ, D65 standard illuminant
/// <https://en.wikipedia.org/wiki/SRGB#From_sRGB_to_CIE_XYZ>
#[no_mangle]
pub extern "C" fn lrgb_to_xyz(pixel: &mut [f32; 3]) {
    *pixel = matmul3t(*pixel, XYZ65_MAT)
}

/// Convert from CIE XYZ to CIE LAB.
/// <https://en.wikipedia.org/wiki/CIELAB_color_space#From_CIEXYZ_to_CIELAB>
#[no_mangle]
pub extern "C" fn xyz_to_lab(pixel: &mut [f32; 3]) {
    // Reverse D65 standard illuminant
    pixel.iter_mut().zip(D65).for_each(|(c, d)| *c /= d);

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

/// Convert from CIE XYZ to OKLAB.
/// <https://bottosson.github.io/posts/oklab/>
#[no_mangle]
pub extern "C" fn xyz_to_oklab(pixel: &mut [f32; 3]) {
    let mut lms = matmul3(*pixel, OKLAB_M1);
    lms.iter_mut().for_each(|c| *c = c.cbrt());
    *pixel = matmul3(lms, OKLAB_M2);
}

/// Convert from CIE LAB to CIE LCH.
/// <https://en.wikipedia.org/wiki/CIELAB_color_space#Cylindrical_model>
#[no_mangle]
pub extern "C" fn lab_to_lch(pixel: &mut [f32; 3]) {
    *pixel = [
        pixel[0],
        (pixel[1].powi(2) + pixel[2].powi(2)).sqrt(),
        pixel[2].atan2(pixel[1]).to_degrees().rem_euclid(360.0),
    ];
}

// ### UP ### }}}

// ### DOWN ### {{{

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
#[no_mangle]
pub extern "C" fn hsv_to_srgb(pixel: &mut [f32; 3]) {
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
#[no_mangle]
pub extern "C" fn lrgb_to_srgb(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| *c = correct_gamma(*c));
}

/// Convert from CIE XYZ to Linear Light RGB.
/// <https://en.wikipedia.org/wiki/SRGB#From_CIE_XYZ_to_sRGB>
#[no_mangle]
pub extern "C" fn xyz_to_lrgb(pixel: &mut [f32; 3]) {
    *pixel = matmul3t(*pixel, XYZ65_MAT_INV)
}

/// Convert from CIE LAB to CIE XYZ.
/// <https://en.wikipedia.org/wiki/CIELAB_color_space#From_CIELAB_to_CIEXYZ>
#[no_mangle]
pub extern "C" fn lab_to_xyz(pixel: &mut [f32; 3]) {
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

    pixel.iter_mut().zip(D65).for_each(|(c, d)| *c *= d);
}

/// Convert from OKLAB to CIE XYZ.
/// <https://bottosson.github.io/posts/oklab/>
#[no_mangle]
pub extern "C" fn oklab_to_xyz(pixel: &mut [f32; 3]) {
    let mut lms = matmul3(*pixel, OKLAB_M2_INV);
    lms.iter_mut().for_each(|c| *c = c.powi(3));
    *pixel = matmul3(lms, OKLAB_M1_INV);
}

/// Convert from CIE LCH to CIE LAB.
/// <https://en.wikipedia.org/wiki/CIELAB_color_space#Cylindrical_model>
#[no_mangle]
pub extern "C" fn lch_to_lab(pixel: &mut [f32; 3]) {
    *pixel = [
        pixel[0],
        pixel[1] * pixel[2].to_radians().cos(),
        pixel[1] * pixel[2].to_radians().sin(),
    ]
}

// DOWN }}}

// ### TESTS ### {{{
#[cfg(test)]
mod tests {
    use super::*;

    const HEX: &str = "#3359F2";
    const IRGB: [u8; 3] = [51, 89, 242];

    // colour-science references
    const SRGB: [f32; 3] = [0.20000000, 0.35000000, 0.95000000];
    const LRGB: [f32; 3] = [0.03310477, 0.10048151, 0.89000541];
    const HSV: [f32; 3] = [0.63333333, 0.78947368, 0.95000000];
    const XYZ: [f32; 3] = [0.21023057, 0.14316084, 0.85856646];
    const LAB: [f32; 3] = [44.68286380, 40.81934559, -80.13283179];
    const LCH: [f32; 3] = [44.68286380, 89.93047151, 296.99411238];
    const OKLAB: [f32; 3] = [0.53893206, -0.01239956, -0.23206808];
    // no OKLCH test as it's the same as CIE LCH

    fn pixcmp_eps(a: [f32; 3], b: [f32; 3], eps: f32) {
        a.iter().zip(b.iter()).for_each(|(ac, bc)| {
            if (ac - bc).abs() > eps {
                panic!(
                    "\n{:.8} {:.8} {:.8}\n{:.8} {:.8} {:.8}\n",
                    a[0], a[1], a[2], b[0], b[1], b[2]
                )
            }
        });
    }

    fn pixcmp(a: [f32; 3], b: [f32; 3]) {
        pixcmp_eps(a, b, 1e-4)
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
    fn oklab_to() {
        let mut pixel = XYZ;
        xyz_to_oklab(&mut pixel);
        pixcmp(pixel, OKLAB);
    }

    #[test]
    fn oklab_from() {
        let mut pixel = OKLAB;
        oklab_to_xyz(&mut pixel);
        pixcmp(pixel, XYZ);
    }

    #[test]
    fn cielab_full() {
        let mut pixel = SRGB;
        convert_space(Space::SRGB, Space::LCH, &mut pixel);
        pixcmp(pixel, LCH);
        convert_space(Space::LCH, Space::SRGB, &mut pixel);
        pixcmp(pixel, SRGB);
    }

    #[test]
    fn oklab_full() {
        let mut pixel = SRGB;
        let mut oklch = OKLAB;
        lab_to_lch(&mut oklch);
        convert_space(Space::SRGB, Space::OKLCH, &mut pixel);
        pixcmp(pixel, oklch);
        convert_space(Space::OKLCH, Space::SRGB, &mut pixel);
        pixcmp(pixel, SRGB);
    }

    #[test]
    fn tree_jump() {
        println!("BEGIN");
        let mut pixel = HSV;
        let mut oklch = OKLAB;
        lab_to_lch(&mut oklch);

        // the hundred conversions gradually decreases accuracy
        let eps = 1e-2;

        // forwards
        println!("HSV -> LCH");
        convert_space(Space::HSV, Space::LCH, &mut pixel);
        pixcmp_eps(pixel, LCH, eps);

        println!("LCH -> OKLCH");
        convert_space(Space::LCH, Space::OKLCH, &mut pixel);
        pixcmp_eps(pixel, oklch, eps);

        println!("OKLCH -> HSV");
        convert_space(Space::OKLCH, Space::HSV, &mut pixel);
        pixcmp_eps(pixel, HSV, eps);

        // backwards
        println!("HSV -> OKLCH");
        convert_space(Space::HSV, Space::OKLCH, &mut pixel);
        pixcmp_eps(pixel, oklch, eps);

        println!("OKLCH -> LCH");
        convert_space(Space::OKLCH, Space::LCH, &mut pixel);
        pixcmp_eps(pixel, LCH, eps);

        println!("LCH -> HSV");
        convert_space(Space::LCH, Space::HSV, &mut pixel);
        pixcmp_eps(pixel, HSV, eps);

        println!("BEGIN");
    }

    #[test]
    fn chunked() {
        let mut pixel = [SRGB];
        convert_space_chunked(Space::SRGB, Space::LCH, &mut pixel);
        pixcmp(pixel[0], LCH)
    }

    #[test]
    fn sliced() {
        let pixel: &mut [f32] = &mut SRGB.clone();
        convert_space_sliced(Space::SRGB, Space::LCH, pixel);
        pixcmp(pixel.try_into().unwrap(), LCH)
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
// ### TESTS ### }}}
