#![warn(missing_docs)]

//! Simple colorspace conversions in pure Rust.
//!
//! All conversions are in-place, except when converting to/from integer and hexadecimal.
//! Formulae are generally taken from their research papers or Wikipedia and validated against
//! colour-science <https://github.com/colour-science/colour>
//!
//! Helmholtz-Kohlrausch compensation formulae sourced from
//! <https://onlinelibrary.wiley.com/doi/10.1002/col.22839>
//!
//! This crate references CIE Standard Illuminant D65 for functions to/from CIE XYZ

use core::cmp::Ordering;
use core::ffi::{c_char, CStr};

// ### CONSTS ### {{{

/// Standard Illuminant D65.
pub const D65: [f32; 3] = [0.9504559270516716, 1.0, 1.0890577507598784];

// CIE LAB
const LAB_DELTA: f32 = 6.0 / 29.0;

// <PQ EOTF Table 4 <https://www.itu.int/rec/R-REC-BT.2100/en>
const PQEOTF_M1: f32 = 0.1593017578125;
const PQEOTF_M2: f32 = 78.84375;
const PQEOTF_C1: f32 = 0.8359375;
const PQEOTF_C2: f32 = 18.8515625;
const PQEOTF_C3: f32 = 18.6875;

// JzAzBz
const JZAZBZ_B: f32 = 1.15;
const JZAZBZ_G: f32 = 0.66;
const JZAZBZ_D: f32 = -0.56;
const JZAZBZ_D0: f32 = 1.6295499532821566 * 1e-11;
const JZAZBZ_P: f32 = 1.7 * PQEOTF_M2;


// ### CONSTS ### }}}

// ### MATRICES ### {{{
// CIE XYZ
const XYZ65_MAT: [[f32; 3]; 3] = [
    [0.4124, 0.3576, 0.1805],
    [0.2126, 0.7152, 0.0722],
    [0.0193, 0.1192, 0.9505],
];

// Original commonly used inverted array
// const XYZ65_MAT_INV: [[f32; 3]; 3] = [
//     [3.2406, -1.5372, -0.4986],
//     [-0.9689, 1.8758, 0.0415],
//     [0.0557, -0.2040, 1.0570],
// ];

// Higher precision invert using numpy. Helps with back conversions
const XYZ65_MAT_INV: [[f32; 3]; 3] = [
    [3.2406254773, -1.5372079722, -0.4986285987],
    [-0.9689307147, 1.8757560609, 0.0415175238],
    [0.0557101204, -0.2040210506, 1.0569959423],
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

// JzAzBz
const JZAZBZ_M1: [[f32; 3]; 3] = [
    [0.41478972, 0.579999, 0.0146480],
    [-0.2015100, 1.120649, 0.0531008],
    [-0.0166008, 0.264800, 0.6684799],
];
const JZAZBZ_M2: [[f32; 3]; 3] = [
    [0.500000, 0.500000, 0.000000],
    [3.524000, -4.066708, 0.542708],
    [0.199076, 1.096799, -1.295875],
];

const JZAZBZ_M1_INV: [[f32; 3]; 3] = [
    [1.9242264358, -1.0047923126, 0.037651404],
    [0.3503167621, 0.7264811939, -0.0653844229],
    [-0.090982811, -0.3127282905, 1.5227665613],
];
const JZAZBZ_M2_INV: [[f32; 3]; 3] = [
    [1., 0.1386050433, 0.0580473162],
    [1., -0.1386050433, -0.0580473162],
    [1., -0.096019242, -0.8118918961],
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

/// Expand gamma of a single value to linear light
#[no_mangle]
pub extern "C" fn expand_gamma(n: f32) -> f32 {
    if n <= 0.04045 {
        n / 12.92
    } else {
        ((n + 0.055) / 1.055_f32).powf(2.4)
    }
}

/// Gamma corrects a single linear light value
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

fn hk_2023_fby(h: f32) -> f32 {
    K_HIGH2022[0] * ((h - 90.0) / 2.0).to_radians().sin().abs() + K_HIGH2022[1]
}

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
pub extern "C" fn hk_high2023(lch: &[f32; 3]) -> f32 {
    (hk_2023_fby(lch[2]) + hk_2023_fr(lch[2])) * lch[1]
}

/// Compensates LCH's L value for the Helmholtz-Kohlrausch effect.
/// High et al 2023 implementation.
#[no_mangle]
pub extern "C" fn hk_high2023_comp(lch: &mut [f32; 3]) {
    lch[0] += HIGH2023_MEAN * (lch[1] / 100.0) - hk_high2023(lch)
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
    /// 1976 UCS with many known flaws. Most other LAB spaces derive from this
    LAB,

    /// CIE L*C*Hab. Lightness, Chroma, Hue
    /// Cylindrical version of CIE L*a*b*.
    LCH,

    /// Oklab <https://bottosson.github.io/posts/oklab/>
    /// 2020 UCS, used in CSS Color Module Level 4
    OKLAB,

    /// Cylindrical version of OKLAB.
    OKLCH,

    /// JzAzBz <https://opg.optica.org/oe/fulltext.cfm?uri=oe-25-13-15131>
    /// 2017 UCS, intended for uniform hue and HDR colors
    JZAZBZ,

    /// Cylindrical version of JzAzBz
    JZCZHZ,
}

impl TryFrom<&str> for Space {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, ()> {
        match value.to_ascii_lowercase().trim() {
            "srgb" | "srgba" => Ok(Space::SRGB),
            "hsv" | "hsva" => Ok(Space::HSV),
            "lrgb" | "lrgba" | "rgb" | "rgba" => Ok(Space::LRGB),
            "xyz" | "xyza" | "cie xyz" => Ok(Space::XYZ),
            "lab" | "laba" | "cie l*a*b*" => Ok(Space::LAB),
            "lch" | "lcha" | "cie l*c*hab" => Ok(Space::LCH),
            "oklab" | "oklaba" => Ok(Space::OKLAB),
            "oklch" | "oklcha" => Ok(Space::OKLCH),
            _ => Err(()),
        }
    }
}

impl core::fmt::Display for Space {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::write(
            f,
            core::format_args!(
                "{}",
                match self {
                    Self::SRGB => "sRGB",
                    Self::HSV => "HSV",
                    Self::LRGB => "RGB",
                    Self::XYZ => "CIE XYZ",
                    Self::LAB => "CIE L*a*b*",
                    Self::LCH => "CIE L*C*Hab",
                    Self::OKLAB => "Oklab",
                    Self::OKLCH => "Oklch",
                    Self::JZAZBZ => "JzAzBz",
                    Self::JZCZHZ => "JzCzHz",
                }
            ),
        )
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
            Space::JZCZHZ => Ordering::Greater,

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
            Space::JZAZBZ => match other {
                Space::JZCZHZ => Ordering::Less,
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
            Space::JZAZBZ => ['j', 'a', 'b'],
            Space::JZCZHZ => ['j', 'c', 'h'],
        }
    }

    /// All color spaces
    pub const ALL: &'static [Space] = &[
        Space::SRGB,
        Space::HSV,
        Space::LRGB,
        Space::XYZ,
        Space::LAB,
        Space::LCH,
        Space::OKLAB,
        Space::OKLCH,
        Space::JZAZBZ,
        Space::JZCZHZ,
    ];

    /// Uniform color spaces
    pub const UCS: &'static [Space] = &[Space::LAB, Space::OKLAB, Space::JZAZBZ];

    /// Uniform color spaces in cylindrical/polar format
    pub const UCS_POLAR: &'static [Space] = &[Space::LCH, Space::OKLCH, Space::JZCZHZ];

    /// RGB/Tristimulus color spaces
    pub const TRI: &'static [Space] = &[Space::SRGB, Space::LRGB, Space::XYZ];
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
            Space::JZAZBZ => {
                pixels.iter_mut().for_each(|pixel| jzazbz_to_xyz(pixel));
                convert_space_chunked(Space::XYZ, to, pixels)
            }
            Space::JZCZHZ => {
                pixels.iter_mut().for_each(|pixel| lch_to_lab(pixel));
                convert_space_chunked(Space::JZAZBZ, to, pixels)
            }
        }
    } else if from < to {
        match from {
            // Endcaps
            Space::HSV => unreachable!(),
            Space::LCH => unreachable!(),
            Space::OKLCH => unreachable!(),
            Space::JZCZHZ => unreachable!(),

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
                Space::JZAZBZ | Space::JZCZHZ => {
                    pixels.iter_mut().for_each(|pixel| xyz_to_jzazbz(pixel));
                    convert_space_chunked(Space::JZAZBZ, to, pixels)
                }
                _ => unreachable!("XYZ tried to promote to {}", to),
            },
            Space::LAB | Space::OKLAB | Space::JZAZBZ => pixels.iter_mut().for_each(|pixel| lab_to_lch(pixel)),
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

// ### FORWARD ### {{{

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

/// Convert CIE XYZ to JzAzBz
/// <https://opg.optica.org/oe/fulltext.cfm?uri=oe-25-13-15131>
#[no_mangle]
pub extern "C" fn xyz_to_jzazbz(pixel: &mut [f32; 3]) {
    let mut lms = matmul3(
        [
            pixel[0] * JZAZBZ_B - (JZAZBZ_B - 1.0) * pixel[2],
            pixel[1] * JZAZBZ_G - (JZAZBZ_G - 1.0) * pixel[0],
            pixel[2],
        ],
        JZAZBZ_M1,
    );

    lms.iter_mut().for_each(|e| {
        *e = (
            (PQEOTF_C1 + PQEOTF_C2 * (*e / 10000.0).powf(PQEOTF_M1)) /
            (1.0 + PQEOTF_C3 * (*e / 10000.0).powf(PQEOTF_M1))
        ).powf(JZAZBZ_P)
    });

    let lab = matmul3(lms, JZAZBZ_M2);

    *pixel = [
        ((1.0 + JZAZBZ_D) * lab[0]) / (1.0 + JZAZBZ_D * lab[0]) - JZAZBZ_D0,
        lab[1],
        lab[2],
    ]
}

// Disabled for now as all the papers are paywalled
/// Convert CIE XYZ to CAM16-UCS
// #[no_mangle]
// pub extern "C" fn xyz_to_cam16ucs(pixel: &mut [f32; 3]) {

// }

// Disabled until the reference fuzziness is resolved
/// Convert LRGB to ICtCp
/// <https://www.itu.int/rec/R-REC-BT.2100/en>
// #[no_mangle]
// pub extern "C" fn lrgb_to_ictcp(pixel: &mut [f32; 3]) {

//     let pq_eotf = |c: &mut f32| *c = 10000.0 *
//         (
//             (c.powf(1.0 / PQEOTF_M2) - PQEOTF_C1).max(0.0) /
//             (PQEOTF_C2 - PQEOTF_C3 * c.powf(1.0 / PQEOTF_M2))
//         )
//         .powf(1.0 / PQEOTF_M1);

//     let pq_eotf_inverse = |c: &mut f32| *c =
//         (
//             (PQEOTF_C1 + PQEOTF_C2 * (*c / 10000.0).powf(PQEOTF_M1)) /
//             (1.0 + PQEOTF_C3 * (*c / 10000.0).powf(PQEOTF_M1))
//         )
//         .powf(PQEOTF_M2);

//     // <https://www.itu.int/rec/R-REC-BT.2020/en>
//     let alpha = 1.09929682680944;
//     let beta = 0.018053968510807;
//     let bt2020 = |e: &mut f32| {
//         *e = if *e < beta {4.5 * *e}
//         else {alpha * e.powf(0.45) - (alpha - 1.0)}
//     };
//     pixel.iter_mut().for_each(|c| bt2020(c));
//     println!("{:?}\n[0.28192627, 0.34645211, 0.88486558]", pixel);

//     let mut lms = matmul3t(*pixel, [
//         [1688.0, 2146.0, 262.0],
//         [683.0, 2951.0, 462.0],
//         [99.0, 309.0, 3688.0],
//     ]);
//     lms.iter_mut().for_each(|c| *c /= 4096.0);

//     // lms prime
//     lms.iter_mut().for_each(|c| pq_eotf_inverse(c));

//     // *pixel = matmul3t(lms, [
//     //     [2048.0, 2048.0, 0.0],
//     //     [6610.0, -13613.0, 7003.0],
//     //     [17933.0, -17390.0, -543.0],
//     // ]);
//     // pixel.iter_mut().for_each(|c| *c /= 4096.0)

//     *pixel = [
//         lms[0] * 0.5 + lms[1] * 0.5,
//         (6610.0 * lms[0] - 13613.0 * lms[1] + 7003.0 * lms[2]) / 4096.0,
//         (17933.0 * lms[0] - 17390.0 * lms[1] - 543.0 * lms[2]) / 4096.0,
//     ];

// }

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

// ### FORWARD ### }}}

// ### BACKWARD ### {{{

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

/// Convert JzAzBz to CIE XYZ
/// <https://opg.optica.org/oe/fulltext.cfm?uri=oe-25-13-15131>
#[no_mangle]
pub extern "C" fn jzazbz_to_xyz(pixel: &mut [f32; 3]) {
    let mut lms = matmul3(
        [
            (pixel[0] + JZAZBZ_D0) / (1.0 + JZAZBZ_D - JZAZBZ_D * (pixel[0] + JZAZBZ_D0)),
            pixel[1],
            pixel[2],
        ],
        JZAZBZ_M2_INV,
    );

    lms.iter_mut().for_each(|c| {
        *c = 10000.0
            * ((PQEOTF_C1 - c.powf(1.0 / JZAZBZ_P))
                / (PQEOTF_C3 * c.powf(1.0 / JZAZBZ_P) - PQEOTF_C2))
                .powf(1.0 / PQEOTF_M1);
    });

    *pixel = matmul3(lms, JZAZBZ_M1_INV);

    pixel[0] = (pixel[0] + (JZAZBZ_B - 1.0) * pixel[2]) / JZAZBZ_B;
    pixel[1] = (pixel[1] + (JZAZBZ_G - 1.0) * pixel[0]) / JZAZBZ_G;
}

// Disabled for now as all the papers are paywalled
/// Convert CAM16-UCS to CIE XYZ
// #[no_mangle]
// pub extern "C" fn cam16ucs_to_xyz(pixel: &mut [f32; 3]) {

// }

/// Convert ICtCp to LRGB
/// <https://www.itu.int/rec/R-REC-BT.2100/en>
// #[no_mangle]
// pub extern "C" fn ictcp_to_lrgb(pixel: &mut [f32; 3]) {

// }

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

// BACKWARD }}}

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
    const JZAZBZ: [f32; 3] = [0.00601244, -0.00145433, -0.01984568];
    // const CAM16UCS: [f32; 3] = [47.84082873, 4.32008711, -34.36538267];
    // const ICTCP: [f32; 3] = [0.07621171, 0.08285557, -0.03831496];

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
        pixcmp_eps(a, b, 1e-5)
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
    fn jzazbz_to() {
        let mut pixel = XYZ;
        xyz_to_jzazbz(&mut pixel);
        pixcmp(pixel, JZAZBZ);
    }

    #[test]
    fn jzazbz_from() {
        let mut pixel = JZAZBZ;
        jzazbz_to_xyz(&mut pixel);
        pixcmp(pixel, XYZ);
    }

    // #[test]
    // fn cam16ucs_to() {
    //     let mut pixel = XYZ;
    //     xyz_to_cam16ucs(&mut pixel);
    //     pixcmp(pixel, CAM16UCS);
    // }

    // #[test]
    // fn cam16ucs_from() {
    //     let mut pixel = CAM16UCS;
    //     cam16ucs_to_xyz(&mut pixel);
    //     pixcmp(pixel, XYZ);
    // }

    // #[test]
    // fn ictcp_to() {
    //     let mut pixel = LRGB;
    //     lrgb_to_ictcp(&mut pixel);
    //     pixcmp(pixel, ICTCP);
    // }

    // #[test]
    // fn ictcp_from() {
    //     let mut pixel = ICTCP;
    //     ictcp_to_lrgb(&mut pixel);
    //     pixcmp(pixel, LRGB);
    // }

    #[test]
    fn tree_jump() {
        println!("BEGIN");
        let mut pixel = HSV;
        let mut oklch = OKLAB;
        lab_to_lch(&mut oklch);
        let mut jzczhz = JZAZBZ;
        lab_to_lch(&mut jzczhz);

        // the hundred conversions gradually decreases accuracy
        let eps = 1e-4;

        // forwards
        println!("HSV -> LCH");
        convert_space(Space::HSV, Space::LCH, &mut pixel);
        pixcmp_eps(pixel, LCH, eps);

        println!("LCH -> OKLCH");
        convert_space(Space::LCH, Space::OKLCH, &mut pixel);
        pixcmp_eps(pixel, oklch, eps);

        println!("OKLCH -> JZCZHZ");
        convert_space(Space::OKLCH, Space::JZCZHZ, &mut pixel);
        pixcmp_eps(pixel, jzczhz, eps);

        println!("JZCZHZ -> HSV");
        convert_space(Space::JZCZHZ, Space::HSV, &mut pixel);
        pixcmp_eps(pixel, HSV, eps);

        // backwards
        println!("HSV -> JZCZHZ");
        convert_space(Space::HSV, Space::JZCZHZ, &mut pixel);
        pixcmp_eps(pixel, jzczhz, eps);

        println!("JZCZHZ -> OKLCH");
        convert_space(Space::JZCZHZ, Space::OKLCH, &mut pixel);
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
