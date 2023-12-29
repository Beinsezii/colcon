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

const SRGBEOTF_ALPHA: f32 = 0.055;
const SRGBEOTF_GAMMA: f32 = 2.4;
// more precise older specs
// const SRGBEOTF_PHI: f32 = 12.9232102;
// const SRGBEOTF_CHI: f32 = 0.0392857;
// const SRGBEOTF_CHI_INV: f32 = 0.0030399;
// less precise but basically official now
const SRGBEOTF_PHI: f32 = 12.92;
const SRGBEOTF_CHI: f32 = 0.04045;
const SRGBEOTF_CHI_INV: f32 = 0.0031308;

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

/// sRGB Electro-Optical Transfer Function
/// <https://en.wikipedia.org/wiki/SRGB#Computing_the_transfer_function>
#[no_mangle]
pub extern "C" fn srgb_eotf(n: f32) -> f32 {
    if n <= SRGBEOTF_CHI {
        n / SRGBEOTF_PHI
    } else {
        ((n + SRGBEOTF_ALPHA) / (1.0 + SRGBEOTF_ALPHA)).powf(SRGBEOTF_GAMMA)
    }
}

/// Inverse sRGB Electro-Optical Transfer Function
/// <https://en.wikipedia.org/wiki/SRGB#Computing_the_transfer_function>
#[no_mangle]
pub extern "C" fn srgb_eotf_inverse(n: f32) -> f32 {
    if n <= SRGBEOTF_CHI_INV {
        n * SRGBEOTF_PHI
    } else {
        (1.0 + SRGBEOTF_ALPHA) * (n.powf(1.0 / SRGBEOTF_GAMMA)) - SRGBEOTF_ALPHA
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
            Space::LAB | Space::OKLAB | Space::JZAZBZ => {
                pixels.iter_mut().for_each(|pixel| lab_to_lch(pixel))
            }
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

/// Convert from sRGB to Linear RGB by applying the sRGB EOTF
/// <https://www.color.org/chardata/rgb/srgb.xalter>
#[no_mangle]
pub extern "C" fn srgb_to_lrgb(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| *c = srgb_eotf(*c));
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
    let mut lms = matmul3t(
        [
            pixel[0] * JZAZBZ_B - (JZAZBZ_B - 1.0) * pixel[2],
            pixel[1] * JZAZBZ_G - (JZAZBZ_G - 1.0) * pixel[0],
            pixel[2],
        ],
        JZAZBZ_M1,
    );

    lms.iter_mut().for_each(|e| {
        let v = *e / 10000.0;
        *e = ((PQEOTF_C1 + PQEOTF_C2 * v.abs().powf(PQEOTF_M1).copysign(v))
            / (1.0 + PQEOTF_C3 * v.abs().powf(PQEOTF_M1).copysign(v)))
        .powf(JZAZBZ_P)
    });

    let lab = matmul3t(lms, JZAZBZ_M2);

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

/// Convert from Linear RGB to sRGB by applying the inverse sRGB EOTF
/// <https://www.color.org/chardata/rgb/srgb.xalter>
#[no_mangle]
pub extern "C" fn lrgb_to_srgb(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| *c = srgb_eotf_inverse(*c));
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
    let mut lms = matmul3t(
        [
            (pixel[0] + JZAZBZ_D0) / (1.0 + JZAZBZ_D - JZAZBZ_D * (pixel[0] + JZAZBZ_D0)),
            pixel[1],
            pixel[2],
        ],
        JZAZBZ_M2_INV,
    );
    lms.iter_mut().for_each(|c| {
        let v = (PQEOTF_C1 - c.powf(1.0 / JZAZBZ_P))
            / (PQEOTF_C3 * c.powf(1.0 / JZAZBZ_P) - PQEOTF_C2);
        *c = 10000.0 * v.abs().powf(1.0 / PQEOTF_M1).copysign(v);
    });

    *pixel = matmul3t(lms, JZAZBZ_M1_INV);

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

    // ### COLOUR-REFS ### {{{

    const SRGB: &'static [[f32; 3]] = &[
        [0.00000000, 0.00000000, 0.00000000],
        [1.00000000, 0.00000000, 0.00000000],
        [0.00000000, 1.00000000, 0.00000000],
        [0.00000000, 0.00000000, 1.00000000],
        [1.00000000, 1.00000000, 0.00000000],
        [0.00000000, 1.00000000, 1.00000000],
        [1.00000000, 0.00000000, 1.00000000],
        [1.00000000, 1.00000000, 1.00000000],
        [5.00000000, 10.00000000, 15.00000000],
        [-5.00000000, -10.00000000, -15.00000000],
    ];
    const LRGB: &'static [[f32; 3]] = &[
        [0.00000000, 0.00000000, 0.00000000],
        [1.00000000, 0.00000000, 0.00000000],
        [0.00000000, 1.00000000, 0.00000000],
        [0.00000000, 0.00000000, 1.00000000],
        [1.00000000, 1.00000000, 0.00000000],
        [0.00000000, 1.00000000, 1.00000000],
        [1.00000000, 0.00000000, 1.00000000],
        [1.00000000, 1.00000000, 1.00000000],
        [42.96599571, 223.82627997, 589.69564509],
        [-0.38699690, -0.77399381, -1.16099071],
    ];
    const HSV: &'static [[f32; 3]] = &[
        [0.00000000, 0.00000000, 0.00000000],
        [0.00000000, 1.00000000, 1.00000000],
        [0.33333333, 1.00000000, 1.00000000],
        [0.66666667, 1.00000000, 1.00000000],
        [0.16666667, 1.00000000, 1.00000000],
        [0.50000000, 1.00000000, 1.00000000],
        [0.83333333, 1.00000000, 1.00000000],
        [0.00000000, 0.00000000, 1.00000000],
        [0.58333333, 0.66666667, 15.00000000],
        [0.08333333, -2.00000000, -5.00000000],
    ];
    const XYZ: &'static [[f32; 3]] = &[
        [0.00000000, 0.00000000, 0.00000000],
        [0.41240000, 0.21260000, 0.01930000],
        [0.35760000, 0.71520000, 0.11920000],
        [0.18050000, 0.07220000, 0.95050000],
        [0.77000000, 0.92780000, 0.13850000],
        [0.53810000, 0.78740000, 1.06970000],
        [0.59290000, 0.28480000, 0.96980000],
        [0.95050000, 1.00000000, 1.08900000],
        [204.19951828, 211.79115169, 588.01504694],
        [-0.64593653, -0.71965944, -1.20325077],
    ];
    const LAB: &'static [[f32; 3]] = &[
        [0.00000000, 0.00000000, 0.00000000],
        [53.23288179, 80.11117774, 67.22370367],
        [87.73703347, -86.18285500, 83.18783466],
        [32.30258667, 79.19808023, -107.85035570],
        [97.13824698, -21.55360786, 94.48949749],
        [91.11652111, -48.07757700, -14.12426716],
        [60.31993366, 98.25632722, -60.82956929],
        [100.00000000, 0.00772827, 0.00353528],
        [675.44970111, 14.25078120, -436.42562428],
        [-650.06570921, 155.94479927, 599.90623227],
    ];
    const LCH: &'static [[f32; 3]] = &[
        [0.00000000, 0.00000000, 0.00000000],
        [53.23288179, 104.57928635, 40.00102571],
        [87.73703347, 119.78188649, 136.01306869],
        [32.30258667, 133.80596077, 306.29106810],
        [97.13824698, 96.91657829, 102.84964820],
        [91.11652111, 50.10936373, 196.37177336],
        [60.31993366, 115.56185503, 328.23873929],
        [100.00000000, 0.00849849, 24.58159697],
        [675.44970111, 436.65823054, 271.87023758],
        [-650.06570921, 619.84374477, 75.42854110],
    ];
    const OKLAB: &'static [[f32; 3]] = &[
        [0.00000000, 0.00000000, 0.00000000],
        [0.62792590, 0.22488760, 0.12580493],
        [0.86645187, -0.23392144, 0.17942177],
        [0.45203295, -0.03235164, -0.31162054],
        [0.96798108, -0.07139347, 0.19848985],
        [0.90541467, -0.14944654, -0.03950465],
        [0.70165739, 0.27462625, -0.16926875],
        [1.00000174, 0.00000229, -0.00011365],
        [5.95611678, -0.42728383, -1.24134000],
        [-0.89252901, 0.04256306, 0.07613246],
    ];
    const OKLCH: &'static [[f32; 3]] = &[
        [0.00000000, 0.00000000, 0.00000000],
        [0.62792590, 0.25768453, 29.22319405],
        [0.86645187, 0.29480741, 142.51117284],
        [0.45203295, 0.31329538, 264.07293384],
        [0.96798108, 0.21093897, 109.78280773],
        [0.90541467, 0.15457971, 194.80686888],
        [0.70165739, 0.32260113, 328.35196366],
        [1.00000174, 0.00011368, 271.15202477],
        [5.95611678, 1.31282005, 251.00593438],
        [-0.89252901, 0.08722251, 60.79193305],
    ];
    const JZAZBZ: &'static [[f32; 3]] = &[
        [0.00000000, 0.00000000, 0.00000000],
        [0.00816620, 0.01616207, 0.01140765],
        [0.01243416, -0.01624847, 0.01656722],
        [0.00478354, -0.00116064, -0.02495001],
        [0.01611356, -0.00372360, 0.02080003],
        [0.01418671, -0.01169390, -0.00544011],
        [0.01049327, 0.01640515, -0.01404140],
        [0.01758021, -0.00002806, -0.00002067],
        [0.22861137, -0.04674604, -0.11322403],
        [-0.78600741, 1933.15497262, 2113.40419865],
    ];
    const JZCZHZ: &'static [[f32; 3]] = &[
        [0.00000000, 0.00000000, 0.00000000],
        [0.00816620, 0.01978250, 35.21553828],
        [0.01243416, 0.02320529, 134.44348766],
        [0.00478354, 0.02497699, 267.33659019],
        [0.01611356, 0.02113070, 100.14952867],
        [0.01418671, 0.01289736, 204.94830520],
        [0.01049327, 0.02159375, 319.43931639],
        [0.01758021, 0.00003485, 216.37796402],
        [0.22861137, 0.12249438, 247.56607117],
        [-0.78600741, 2864.18670045, 47.55048725],
    ];

    // ### COLOUR-REFS ### }}}

    fn pix_cmp(input: &[[f32; 3]], reference: &[[f32; 3]], epsilon: f32, skips: &'static [usize]) {
        for (n, (i, r)) in input.iter().zip(reference.iter()).enumerate() {
            if skips.contains(&n) {
                continue;
            }
            for (a, b) in i.iter().zip(r.iter()) {
                if (a - b).abs() > epsilon || !a.is_finite() || !b.is_finite() {
                    panic!(
                        "\nA{n}: {:.8} {:.8} {:.8}\nB{n}: {:.8} {:.8} {:.8}\n",
                        i[0], i[1], i[2], r[0], r[1], r[2]
                    )
                }
            }
        }
    }

    fn func_cmp_full(
        input: &[[f32; 3]],
        reference: &[[f32; 3]],
        function: extern "C" fn(&mut [f32; 3]),
        epsilon: f32,
        skips: &'static [usize],
    ) {
        let mut input = input.to_owned();
        input.iter_mut().for_each(|p| function(p));
        pix_cmp(&input, reference, epsilon, skips);
    }

    fn func_cmp(
        input: &[[f32; 3]],
        reference: &[[f32; 3]],
        function: extern "C" fn(&mut [f32; 3]),
    ) {
        func_cmp_full(input, reference, function, 1e-3, &[])
    }

    fn conv_cmp_full(
        input_space: Space,
        input: &[[f32; 3]],
        reference_space: Space,
        reference: &[[f32; 3]],
        epsilon: f32,
        skips: &'static [usize],
    ) {
        let mut input = input.to_owned();
        convert_space_chunked(input_space, reference_space, &mut input);
        pix_cmp(&input, reference, epsilon, skips)
    }

    fn conv_cmp(
        input_space: Space,
        input: &[[f32; 3]],
        reference_space: Space,
        reference: &[[f32; 3]],
    ) {
        conv_cmp_full(input_space, input, reference_space, reference, 1e-1, &[0, 7, 8, 9])
    }

    #[test]
    fn hsv_forwards() {
        func_cmp(SRGB, HSV, srgb_to_hsv)
    }
    #[test]
    fn hsv_backwards() {
        func_cmp(HSV, SRGB, hsv_to_srgb)
    }

    #[test]
    fn lrgb_forwards() {
        func_cmp(SRGB, LRGB, srgb_to_lrgb)
    }
    #[test]
    fn lrgb_backwards() {
        func_cmp(LRGB, SRGB, lrgb_to_srgb)
    }

    #[test]
    fn xyz_forwards() {
        func_cmp(LRGB, XYZ, lrgb_to_xyz)
    }
    #[test]
    fn xyz_backwards() {
        func_cmp(XYZ, LRGB, xyz_to_lrgb)
    }

    #[test]
    fn lab_forwards() {
        func_cmp(XYZ, LAB, xyz_to_lab)
    }
    #[test]
    fn lab_backwards() {
        func_cmp(LAB, XYZ, lab_to_xyz)
    }

    #[test]
    fn lch_forwards() {
        func_cmp(LAB, LCH, lab_to_lch)
    }
    #[test]
    fn lch_backwards() {
        func_cmp(LCH, LAB, lch_to_lab)
    }

    #[test]
    fn oklab_forwards() {
        func_cmp(XYZ, OKLAB, xyz_to_oklab)
    }
    #[test]
    fn oklab_backwards() {
        func_cmp(OKLAB, XYZ, oklab_to_xyz)
    }

    // Lower epsilon because of the extremely wide gamut creating tiny values
    #[test]
    fn jzazbz_forwards() {
        func_cmp_full(XYZ, JZAZBZ, xyz_to_jzazbz, 2e-1, &[])
    }
    #[test]
    fn jzazbz_backwards() {
        func_cmp_full(JZAZBZ, XYZ, jzazbz_to_xyz, 2e-1, &[])
    }

    #[test]
    fn tree_jump() {
        // forwards
        println!("HSV -> LCH");
        conv_cmp(Space::HSV, HSV, Space::LCH, LCH);

        println!("LCH -> OKLCH");
        conv_cmp(Space::LCH, LCH, Space::OKLCH, OKLCH);

        println!("OKLCH -> JZCZHZ");
        conv_cmp(Space::OKLCH, OKLCH, Space::JZCZHZ, JZCZHZ);

        println!("JZCZHZ -> HSV");
        conv_cmp(Space::JZCZHZ, JZCZHZ, Space::HSV, HSV);

        // backwards
        println!("HSV -> JZCZHZ");
        conv_cmp(Space::HSV, HSV, Space::JZCZHZ, JZCZHZ);

        println!("JZCZHZ -> OKLCH");
        conv_cmp(Space::JZCZHZ, JZCZHZ, Space::OKLCH, OKLCH);

        println!("OKLCH -> LCH");
        conv_cmp(Space::OKLCH, OKLCH, Space::LCH, LCH);

        // add 1 to skip because the hue wraps from 0.0000 to 0.9999
        // fuck you precision
        println!("LCH -> HSV");
        conv_cmp_full(Space::LCH, LCH, Space::HSV, HSV, 1e-1, &[0, 1, 7, 8, 9]);
    }

    #[test]
    fn sliced() {
        let mut pixel: Vec<f32> = SRGB.iter().fold(Vec::new(), |mut acc, it| {
            acc.extend_from_slice(it);
            acc
        });
        convert_space_sliced(Space::SRGB, Space::LCH, &mut pixel);
        pix_cmp(
            &pixel
                .chunks_exact(3)
                .map(|c| c.try_into().unwrap())
                .collect::<Vec<[f32; 3]>>(),
            LCH,
            1e-1,
            &[],
        )
    }

    #[test]
    fn irgb_to() {
        assert_eq!(IRGB, srgb_to_irgb([0.2, 0.35, 0.95]))
    }

    #[test]
    fn irgb_from() {
        let mut srgb = irgb_to_srgb(IRGB);
        srgb.iter_mut()
            .for_each(|c| *c = (*c * 100.0).round() / 100.0);
        assert_eq!([0.2, 0.35, 0.95], srgb)
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
