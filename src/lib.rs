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

fn spowf(n: f32, power: f32) -> f32 {
    n.abs().powf(power).copysign(n)
}

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

// ICtCp
const ICTCP_M1: [[f32; 3]; 3] = [
    [1688.0 / 4096.0, 2146.0 / 4096.0, 262.0 / 4096.0],
    [683.0 / 4096.0, 2951.0 / 4096.0, 462.0 / 4096.0],
    [99.0 / 4096.0, 309.0 / 4096.0, 3688.0 / 4096.0],
];
const ICTCP_M2: [[f32; 3]; 3] = [
    [2048.0 / 4096.0, 2048.0 / 4096.0, 0.0 / 4096.0],
    [6610.0 / 4096.0, -13613.0 / 4096.0, 7003.0 / 4096.0],
    [17933.0 / 4096.0, -17390.0 / 4096.0, -543.0 / 4096.0],
];

const ICTCP_M1_INV: [[f32; 3]; 3] = [
    [3.4366066943, -2.5064521187, 0.0698454243],
    [-0.7913295556, 1.9836004518, -0.1922708962],
    [-0.0259498997, -0.0989137147, 1.1248636144],
];
const ICTCP_M2_INV: [[f32; 3]; 3] = [
    [1., 0.008609037, 0.111029625],
    [1., -0.008609037, -0.111029625],
    [1., 0.5600313357, -0.320627175],
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

// ### TRANSFER FUNCTIONS ### {{{

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

/// <https://www.itu.int/rec/R-REC-BT.2100/en> Table 4 "Reference PQ EOTF"
fn pq_eotf_common(e: f32, m2: f32) -> f32 {
    let y = spowf(
        (spowf(e, 1.0 / m2) - PQEOTF_C1).max(0.0) / (PQEOTF_C2 - PQEOTF_C3 * spowf(e, 1.0 / m2)),
        1.0 / PQEOTF_M1,
    );
    10000.0 * y
}

/// <https://www.itu.int/rec/R-REC-BT.2100/en> Table 4 "Reference PQ OETF"
fn pq_oetf_common(f: f32, m2: f32) -> f32 {
    let y = f / 10000.0;
    spowf(
        (PQEOTF_C1 + PQEOTF_C2 * spowf(y, PQEOTF_M1)) / (1.0 + PQEOTF_C3 * spowf(y, PQEOTF_M1)),
        m2,
    )
}

/// Dolby Perceptual Quantizer Electro-Optical Transfer Function primarily used for ICtCP
/// <https://www.itu.int/rec/R-REC-BT.2100/en> Table 4 "Reference PQ EOTF"
#[no_mangle]
pub extern "C" fn pq_eotf(e: f32) -> f32 {
    pq_eotf_common(e, PQEOTF_M2)
}

/// Dolby Perceptual Quantizer Optical-Electro Transfer Function primarily used for ICtCP
/// <https://www.itu.int/rec/R-REC-BT.2100/en> Table 4 "Reference PQ OETF"
#[no_mangle]
pub extern "C" fn pq_oetf(f: f32) -> f32 {
    pq_oetf_common(f, PQEOTF_M2)
}

/// Dolby Perceptual Quantizer Electro-Optical Transfer Function modified for JzAzBz
/// Replaced PQEOTF_M2 with JZAZBZ_P
/// <https://www.itu.int/rec/R-REC-BT.2100/en> Table 4 "Reference PQ EOTF"
#[no_mangle]
pub extern "C" fn pqz_eotf(e: f32) -> f32 {
    pq_eotf_common(e, JZAZBZ_P)
}

/// Dolby Perceptual Quantizer Optical-Electro Transfer Function modified for JzAzBz
/// Replaced PQEOTF_M2 with JZAZBZ_P
/// <https://www.itu.int/rec/R-REC-BT.2100/en> Table 4 "Reference PQ OETF"
#[no_mangle]
pub extern "C" fn pqz_oetf(f: f32) -> f32 {
    pq_oetf_common(f, JZAZBZ_P)
}

// ### TRANSFER FUNCTIONS ### }}}

// ### Helmholtz-Kohlrausch ### {{{

/// Extended K-values from High et al 2021/2022
const K_HIGH2022: [f32; 4] = [0.1644, 0.0603, 0.1307, 0.0060];

/// Mean value of the HK delta for CIE LCH(ab), High et al 2023 implementation.
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

/// Compensates CIE LCH's L value for the Helmholtz-Kohlrausch effect.
/// High et al 2023 implementation.
#[no_mangle]
pub extern "C" fn hk_high2023_comp(lch: &mut [f32; 3]) {
    lch[0] += (HIGH2023_MEAN - hk_high2023(lch)) * (lch[1] / 100.0)
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

    /// CIE LAB. Lightness, red/green chromacity, yellow/blue chromacity.
    /// 1976 UCS with many known flaws. Most other LAB spaces derive from this
    CIELAB,

    /// CIE LCH(ab). Lightness, Chroma, Hue
    /// Cylindrical version of CIE LAB.
    CIELCH,

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
            "srgb" => Ok(Space::SRGB),
            "hsv" => Ok(Space::HSV),
            "lrgb" | "rgb" => Ok(Space::LRGB),
            "xyz" | "cie xyz" | "ciexyz" => Ok(Space::XYZ),
            // extra values so you can move to/from str
            "lab" | "cie lab" | "cielab" => Ok(Space::CIELAB),
            "lch" | "cie lch" | "cielch" => Ok(Space::CIELCH),
            "oklab" => Ok(Space::OKLAB),
            "oklch" => Ok(Space::OKLCH),
            "jzazbz" => Ok(Space::JZAZBZ),
            "jzczhz" => Ok(Space::JZCZHZ),
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
                    Self::CIELAB => "CIE LAB",
                    Self::CIELCH => "CIE LCH",
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
            Space::CIELCH => Ordering::Greater,
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
            Space::CIELAB => match other {
                Space::CIELCH => Ordering::Less,
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
            Space::CIELAB => ['l', 'a', 'b'],
            Space::CIELCH => ['l', 'c', 'h'],
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
        Space::CIELAB,
        Space::CIELCH,
        Space::OKLAB,
        Space::OKLCH,
        Space::JZAZBZ,
        Space::JZCZHZ,
    ];

    /// Uniform color spaces
    pub const UCS: &'static [Space] = &[Space::CIELAB, Space::OKLAB, Space::JZAZBZ];

    /// Uniform color spaces in cylindrical/polar format
    pub const UCS_POLAR: &'static [Space] = &[Space::CIELCH, Space::OKLCH, Space::JZCZHZ];

    /// RGB/Tristimulus color spaces
    pub const TRI: &'static [Space] = &[Space::SRGB, Space::LRGB, Space::XYZ];

    // ### GENREATED QUANTILE FNS ### {{{

    /// Retrieves the 0 quantile for mapping a given Space back to SRGB.
    /// This is useful for things like creating adjustable values in Space
    /// that represent most of the SRGB range without clipping.
    /// Wrapping Hue values are set to f32::INFINITY
    pub const fn srgb_quant0(&self) -> [f32; 3] {
        match self {
            &Space::SRGB => [0.0, 0.0, 0.0],
            &Space::HSV => [f32::INFINITY, 0.0, 0.0],
            &Space::LRGB => [0.0, 0.0, 0.0],
            &Space::XYZ => [0.0, 0.0, 0.0],
            &Space::CIELAB => [0.0, -86.18286, -107.85035],
            &Space::CIELCH => [0.0, 0.0, f32::INFINITY],
            &Space::OKLAB => [0.0, -0.23392147, -0.31162056],
            &Space::OKLCH => [0.0, 0.0, f32::INFINITY],
            &Space::JZAZBZ => [0.0, -0.016248252, -0.02494995],
            &Space::JZCZHZ => [0.0, 0.0, f32::INFINITY],
        }
    }
    /// Retrieves the 1 quantile for mapping a given Space back to SRGB.
    /// This is useful for things like creating adjustable values in Space
    /// that represent most of the SRGB range without clipping.
    /// Wrapping Hue values are set to f32::INFINITY
    pub const fn srgb_quant1(&self) -> [f32; 3] {
        match self {
            &Space::SRGB => [0.01, 0.01, 0.01],
            &Space::HSV => [f32::INFINITY, 0.100000024, 0.21],
            &Space::LRGB => [0.0007739938, 0.0007739938, 0.0007739938],
            &Space::XYZ => [0.017851105, 0.013837883, 0.008856518],
            &Space::CIELAB => [11.849316, -76.75052, -92.486176],
            &Space::CIELCH => [11.849316, 7.061057, f32::INFINITY],
            &Space::OKLAB => [0.24800068, -0.20801869, -0.26735336],
            &Space::OKLCH => [0.24800068, 0.020308746, f32::INFINITY],
            &Space::JZAZBZ => [0.00098745, -0.014176477, -0.021382865],
            &Space::JZCZHZ => [0.00098745, 0.0010760628, f32::INFINITY],
        }
    }
    /// Retrieves the 5 quantile for mapping a given Space back to SRGB.
    /// This is useful for things like creating adjustable values in Space
    /// that represent most of the SRGB range without clipping.
    /// Wrapping Hue values are set to f32::INFINITY
    pub const fn srgb_quant5(&self) -> [f32; 3] {
        match self {
            &Space::SRGB => [0.05, 0.05, 0.05],
            &Space::HSV => [f32::INFINITY, 0.22500001, 0.37],
            &Space::LRGB => [0.0039359396, 0.0039359396, 0.0039359396],
            &Space::XYZ => [0.051534407, 0.03933429, 0.024504842],
            &Space::CIELAB => [23.45013, -63.685505, -73.55361],
            &Space::CIELCH => [23.45013, 16.198933, f32::INFINITY],
            &Space::OKLAB => [0.35184953, -0.17341, -0.21155143],
            &Space::OKLCH => [0.35184953, 0.045713905, f32::INFINITY],
            &Space::JZAZBZ => [0.0022844237, -0.011591051, -0.01684369],
            &Space::JZCZHZ => [0.0022844237, 0.0026996532, f32::INFINITY],
        }
    }
    /// Retrieves the 10 quantile for mapping a given Space back to SRGB.
    /// This is useful for things like creating adjustable values in Space
    /// that represent most of the SRGB range without clipping.
    /// Wrapping Hue values are set to f32::INFINITY
    pub const fn srgb_quant10(&self) -> [f32; 3] {
        match self {
            &Space::SRGB => [0.1, 0.1, 0.1],
            &Space::HSV => [f32::INFINITY, 0.31868133, 0.46],
            &Space::LRGB => [0.010022826, 0.010022826, 0.010022826],
            &Space::XYZ => [0.08368037, 0.06354216, 0.042235486],
            &Space::CIELAB => [30.28909, -53.02161, -59.601402],
            &Space::CIELCH => [30.28909, 23.07717, f32::INFINITY],
            &Space::OKLAB => [0.41310564, -0.14668256, -0.17003474],
            &Space::OKLCH => [0.41310564, 0.064760715, f32::INFINITY],
            &Space::JZAZBZ => [0.003282838, -0.009647734, -0.013499062],
            &Space::JZCZHZ => [0.003282838, 0.0040425155, f32::INFINITY],
        }
    }
    /// Retrieves the 90 quantile for mapping a given Space back to SRGB.
    /// This is useful for things like creating adjustable values in Space
    /// that represent most of the SRGB range without clipping.
    /// Wrapping Hue values are set to f32::INFINITY
    pub const fn srgb_quant90(&self) -> [f32; 3] {
        match self {
            &Space::SRGB => [0.9, 0.9, 0.9],
            &Space::HSV => [f32::INFINITY, 0.95604396, 0.97],
            &Space::LRGB => [0.78741235, 0.78741235, 0.78741235],
            &Space::XYZ => [0.5299013, 0.66003364, 0.7995572],
            &Space::CIELAB => [84.99813, 66.65242, 63.00903],
            &Space::CIELCH => [84.99813, 94.23378, f32::INFINITY],
            &Space::OKLAB => [0.8573815, 0.17646486, 0.1382165],
            &Space::OKLCH => [0.8573815, 0.24145529, f32::INFINITY],
            &Space::JZAZBZ => [0.012802434, 0.010417011, 0.012386818],
            &Space::JZCZHZ => [0.012802434, 0.017864665, f32::INFINITY],
        }
    }
    /// Retrieves the 95 quantile for mapping a given Space back to SRGB.
    /// This is useful for things like creating adjustable values in Space
    /// that represent most of the SRGB range without clipping.
    /// Wrapping Hue values are set to f32::INFINITY
    pub const fn srgb_quant95(&self) -> [f32; 3] {
        match self {
            &Space::SRGB => [0.95, 0.95, 0.95],
            &Space::HSV => [f32::INFINITY, 0.9814815, 0.99],
            &Space::LRGB => [0.8900055, 0.8900055, 0.8900055],
            &Space::XYZ => [0.60459995, 0.7347975, 0.8984568],
            &Space::CIELAB => [88.676025, 74.66978, 72.00491],
            &Space::CIELCH => [88.676025, 103.532455, f32::INFINITY],
            &Space::OKLAB => [0.8893366, 0.20846832, 0.15682206],
            &Space::OKLCH => [0.8893366, 0.26152557, f32::INFINITY],
            &Space::JZAZBZ => [0.013801003, 0.012809383, 0.0145695675],
            &Space::JZCZHZ => [0.013801003, 0.019513208, f32::INFINITY],
        }
    }
    /// Retrieves the 99 quantile for mapping a given Space back to SRGB.
    /// This is useful for things like creating adjustable values in Space
    /// that represent most of the SRGB range without clipping.
    /// Wrapping Hue values are set to f32::INFINITY
    pub const fn srgb_quant99(&self) -> [f32; 3] {
        match self {
            &Space::SRGB => [0.99, 0.99, 0.99],
            &Space::HSV => [f32::INFINITY, 1.0, 1.0],
            &Space::LRGB => [0.9774019, 0.9774019, 0.9774019],
            &Space::XYZ => [0.73368907, 0.84591866, 0.99041635],
            &Space::CIELAB => [93.70696, 85.19584, 82.5843],
            &Space::CIELCH => [93.70696, 116.85265, f32::INFINITY],
            &Space::OKLAB => [0.93844295, 0.246955, 0.17727482],
            &Space::OKLCH => [0.93844295, 0.28863373, f32::INFINITY],
            &Space::JZAZBZ => [0.015377459, 0.01548976, 0.017363891],
            &Space::JZCZHZ => [0.015377459, 0.021917712, f32::INFINITY],
        }
    }
    /// Retrieves the 100 quantile for mapping a given Space back to SRGB.
    /// This is useful for things like creating adjustable values in Space
    /// that represent most of the SRGB range without clipping.
    /// Wrapping Hue values are set to f32::INFINITY
    pub const fn srgb_quant100(&self) -> [f32; 3] {
        match self {
            &Space::SRGB => [1.0, 1.0, 1.0],
            &Space::HSV => [f32::INFINITY, 1.0, 1.0],
            &Space::LRGB => [1.0, 1.0, 1.0],
            &Space::XYZ => [0.9505, 1.0, 1.089],
            &Space::CIELAB => [100.0, 98.25632, 94.489494],
            &Space::CIELCH => [100.0, 133.80594, f32::INFINITY],
            &Space::OKLAB => [1.0000017, 0.2762708, 0.19848984],
            &Space::OKLCH => [1.0000017, 0.32260108, f32::INFINITY],
            &Space::JZAZBZ => [0.01758024, 0.017218411, 0.020799855],
            &Space::JZCZHZ => [0.01758024, 0.024976956, f32::INFINITY],
        }
    }

    // ### GENREATED QUANTILE FNS ### }}}
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
            Space::CIELAB => {
                pixels.iter_mut().for_each(|pixel| lab_to_xyz(pixel));
                convert_space_chunked(Space::XYZ, to, pixels)
            }
            Space::CIELCH => {
                pixels.iter_mut().for_each(|pixel| lch_to_lab(pixel));
                convert_space_chunked(Space::CIELAB, to, pixels)
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
            Space::CIELCH => unreachable!(),
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
                Space::CIELAB | Space::CIELCH => {
                    pixels.iter_mut().for_each(|pixel| xyz_to_lab(pixel));
                    convert_space_chunked(Space::CIELAB, to, pixels)
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
            Space::CIELAB | Space::OKLAB | Space::JZAZBZ => {
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

// ### Str2Col ### {{{
fn rm_paren<'a>(s: &'a str) -> &'a str {
    if let (Some(f), Some(l)) = (s.chars().next(), s.chars().last()) {
        if ['(', '[', '{'].contains(&f) && [')', ']', '}'].contains(&l) {
            return &s[1..(s.len() - 1)];
        }
    }
    s
}

/// Convert a string into a space/array combo.
/// Can separate with parentheses, spaces, ';', ':', or ','
/// Ex: "0.2, 0.5, 0.6", "lch: 50 20 120" "oklab(0.2 0.6 90)"
///
/// Does not support alpha channel.
pub fn str2col(mut s: &str) -> Option<(Space, [f32; 3])> {
    s = rm_paren(s.trim());
    let mut space = Space::SRGB;

    // Return hex if valid
    if let Ok(irgb) = hex_to_irgb(s) {
        return Some((space, irgb_to_srgb(irgb)));
    }

    let seps = [',', ':', ';'];

    // Find Space at front then trim
    if let Some(i) =
        s.find(|c: char| c.is_whitespace() || seps.contains(&c) || ['(', '[', '{'].contains(&c))
    {
        if let Ok(sp) = Space::try_from(&s[..i]) {
            space = sp;
            s = rm_paren(
                s[i..].trim_start_matches(|c: char| c.is_whitespace() || seps.contains(&c)),
            );
        }
    }

    // Split by separators + whitespace and parse
    let splits = s
        .split(|c: char| c.is_whitespace() || seps.contains(&c))
        .filter_map(|s| match s.is_empty() {
            true => None,
            false => Some(s.parse::<f32>().ok()),
        })
        .collect::<Vec<Option<f32>>>();

    // Return floats if all 3 valid
    if splits.len() == 3 {
        if let (Some(a), Some(b), Some(c)) = (splits[0], splits[1], splits[2]) {
            return Some((space, [a, b, c]));
        }
    }

    None
}

/// Convert a string into a pixel of the requested Space
/// Shorthand for str2col() -> convert_space()
pub fn str2space(s: &str, to: Space) -> Option<[f32; 3]> {
    str2col(s).map(|(from, mut col)| {
        convert_space(from, to, &mut col);
        col
    })
}
// ### Str2Col ### }}}

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

    lms.iter_mut().for_each(|e| *e = pqz_oetf(*e));

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

/// Convert LRGB to ICtCp. Unvalidated, WIP
/// <https://www.itu.int/rec/R-REC-BT.2100/en>
// #[no_mangle]
pub extern "C" fn _lrgb_to_ictcp(pixel: &mut [f32; 3]) {
    // <https://www.itu.int/rec/R-REC-BT.2020/en>
    // let alpha = 1.09929682680944;
    // let beta = 0.018053968510807;
    // let bt2020 = |e: &mut f32| {
    //     *e = if *e < beta {4.5 * *e}
    //     else {alpha * e.powf(0.45) - (alpha - 1.0)}
    // };
    // pixel.iter_mut().for_each(|c| bt2020(c));

    let mut lms = matmul3t(*pixel, ICTCP_M1);
    // lms prime
    lms.iter_mut().for_each(|c| *c = pq_oetf(*c));
    *pixel = matmul3t(lms, ICTCP_M2);
}

/// Converts an LAB based space to a cylindrical representation.
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

    lms.iter_mut().for_each(|c| *c = pqz_eotf(*c));

    *pixel = matmul3t(lms, JZAZBZ_M1_INV);

    pixel[0] = (pixel[0] + (JZAZBZ_B - 1.0) * pixel[2]) / JZAZBZ_B;
    pixel[1] = (pixel[1] + (JZAZBZ_G - 1.0) * pixel[0]) / JZAZBZ_G;
}

// Disabled for now as all the papers are paywalled
/// Convert CAM16-UCS to CIE XYZ
// #[no_mangle]
// pub extern "C" fn cam16ucs_to_xyz(pixel: &mut [f32; 3]) {

// }

/// Convert ICtCp to LRGB. Unvalidated, WIP
/// <https://www.itu.int/rec/R-REC-BT.2100/en>
// #[no_mangle]
pub extern "C" fn _ictcp_to_lrgb(pixel: &mut [f32; 3]) {
    // lms prime
    let mut lms = matmul3t(*pixel, ICTCP_M2_INV);
    // non-prime lms
    lms.iter_mut().for_each(|c| *c = pq_eotf(*c));
    *pixel = matmul3t(lms, ICTCP_M1_INV);
}

/// Retrieves an LAB based space from its cylindrical representation.
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

    const _ICTCP2: &'static [[f32; 3]] = &[
        [0.00000073, -0.00000000, 0.00000000],
        [0.08575747, -0.02634122, 0.09894511],
        [0.13074534, -0.11285245, -0.02347905],
        [0.06172962, 0.10943968, -0.05098289],
        [0.14470233, -0.10333260, 0.01679849],
        [0.13713260, -0.00256371, -0.03877447],
        [0.09966927, 0.10169935, 0.04737088],
        [0.14994586, -0.00000057, 0.00000710],
        [0.58829375, 0.11054605, -0.09104248],
        [140.00646866, -38.81567400, 54.38089910],
    ];

    // ### COLOUR-REFS ### }}}

    // ### Comparison FNs ### {{{
    fn pix_cmp(input: &[[f32; 3]], reference: &[[f32; 3]], epsilon: f32, skips: &'static [usize]) {
        let mut err = String::new();
        let mut cum_err = 0.0;
        for (n, (i, r)) in input.iter().zip(reference.iter()).enumerate() {
            if skips.contains(&n) {
                continue;
            }
            for (a, b) in i.iter().zip(r.iter()) {
                if (a - b).abs() > epsilon || !a.is_finite() || !b.is_finite() {
                    let dev = i
                        .iter()
                        .zip(r.iter())
                        .map(|(ix, rx)| ((ix - rx) + 1.0).abs().powi(2))
                        .sum::<f32>();
                    err.push_str(&format!(
                        "\nA{n}: {:.8} {:.8} {:.8}\nB{n}: {:.8} {:.8} {:.8}\nERR²: {}\n",
                        i[0], i[1], i[2], r[0], r[1], r[2], dev
                    ));
                    if dev.is_finite() {
                        cum_err += dev
                    };
                    break;
                }
            }
        }
        if !err.is_empty() {
            panic!("{}\nCUM ERR²: {}", err, cum_err)
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
        conv_cmp_full(
            input_space,
            input,
            reference_space,
            reference,
            1e-2,
            &[0, 7],
        )
    }
    // ### Comparison FNs ### }}}

    // ### Single FN Accuracy ### {{{
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

    // ICtCp development tests.
    // Inversion test in absence of solid reference
    #[test]
    fn ictcp_inversion() {
        let mut pixel = LRGB.to_owned();
        pixel.iter_mut().for_each(|p| _lrgb_to_ictcp(p));
        pixel.iter_mut().for_each(|p| _ictcp_to_lrgb(p));
        pix_cmp(&pixel, LRGB, 1e-1, &[]);
    }
    // Disable reference tests for public commits
    //
    // #[test]
    // fn ictcp_forwards() {
    //     func_cmp(LRGB, _ICTCP2, _lrgb_to_ictcp)
    // }
    // #[test]
    // fn ictcp_backwards() {
    //     func_cmp(_ICTCP2, LRGB, _ictcp_to_lrgb)
    // }
    // ### Single FN Accuracy ### }}}

    /// ### Other Tests ### {{{
    #[test]
    fn tree_jump() {
        // forwards
        println!("HSV -> LCH");
        conv_cmp(Space::HSV, HSV, Space::CIELCH, LCH);

        println!("LCH -> OKLCH");
        conv_cmp(Space::CIELCH, LCH, Space::OKLCH, OKLCH);

        println!("OKLCH -> JZCZHZ");
        conv_cmp_full(Space::OKLCH, OKLCH, Space::JZCZHZ, JZCZHZ, 2e-1, &[0, 7]);

        println!("JZCZHZ -> HSV");
        conv_cmp(Space::JZCZHZ, JZCZHZ, Space::HSV, HSV);

        // backwards
        println!("HSV -> JZCZHZ");
        conv_cmp_full(Space::HSV, HSV, Space::JZCZHZ, JZCZHZ, 2e-1, &[0, 7]);

        println!("JZCZHZ -> OKLCH");
        conv_cmp_full(Space::JZCZHZ, JZCZHZ, Space::OKLCH, OKLCH, 1e-1, &[0, 7]);

        println!("OKLCH -> LCH");
        conv_cmp(Space::OKLCH, OKLCH, Space::CIELCH, LCH);

        // add 1 to skip because the hue wraps from 0.0000 to 0.9999
        // fuck you precision
        println!("LCH -> HSV");
        conv_cmp_full(Space::CIELCH, LCH, Space::HSV, HSV, 1e-1, &[0, 1, 7]);
    }

    #[test]
    fn sliced() {
        let mut pixel: Vec<f32> = SRGB.iter().fold(Vec::new(), |mut acc, it| {
            acc.extend_from_slice(it);
            acc
        });
        convert_space_sliced(Space::SRGB, Space::CIELCH, &mut pixel);
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
    fn nan_checks() {
        let it = [1e+3, -1e+3, 1e-3, -1e-3];
        let fns: &[(&'static str, extern "C" fn(&mut [f32; 3]))] = &[
            ("hsv_forwards", srgb_to_hsv),
            ("hsv_backwards", hsv_to_srgb),
            ("lrgb_forwards", srgb_to_lrgb),
            ("lrgb_backwards", lrgb_to_srgb),
            ("xyz_forwards", lrgb_to_xyz),
            ("xyz_backwards", xyz_to_lrgb),
            ("lab_forwards", xyz_to_lab),
            ("lab_backwards", lab_to_xyz),
            ("lch_forwards", lab_to_lch),
            ("lch_backwards", lch_to_lab),
            ("oklab_forwards", xyz_to_oklab),
            ("oklab_backwards", oklab_to_xyz),
            // ("jzazbz_forwards", xyz_to_jzazbz), // ugh
            ("jzazbz_backwards", jzazbz_to_xyz),
        ];
        for (label, func) in fns {
            for a in it.iter() {
                for b in it.iter() {
                    for c in it.iter() {
                        let from: [f32; 3] = [*a, *b, *c];
                        let mut to = from;
                        func(&mut to);
                        if to.iter().any(|c| !c.is_finite()) {
                            panic!("{} : {:?} -> {:?}", label, from, to);
                        }
                    }
                }
            }
        }
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
                    convert_space(Space::SRGB, Space::CIELCH, &mut pixel);
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

    #[test]
    fn space_strings() {
        for space in Space::ALL {
            assert_eq!(Ok(*space), Space::try_from(space.to_string().as_str()))
        }
    }

    /// ### Other Tests ### }}}

    // ### Str2Col ### {{{
    #[test]
    fn str2col_base() {
        assert_eq!(
            str2col("0.2, 0.5, 0.6"),
            Some((Space::SRGB, [0.2, 0.5, 0.6]))
        )
    }

    #[test]
    fn str2col_base_tight() {
        assert_eq!(str2col("0.2,0.5,0.6"), Some((Space::SRGB, [0.2, 0.5, 0.6])))
    }

    #[test]
    fn str2col_base_lop() {
        assert_eq!(
            str2col("0.2,0.5, 0.6"),
            Some((Space::SRGB, [0.2, 0.5, 0.6]))
        )
    }

    #[test]
    fn str2col_base_bare() {
        assert_eq!(str2col("0.2 0.5 0.6"), Some((Space::SRGB, [0.2, 0.5, 0.6])))
    }

    #[test]
    fn str2col_base_bare_fat() {
        assert_eq!(
            str2col("  0.2   0.5     0.6 "),
            Some((Space::SRGB, [0.2, 0.5, 0.6]))
        )
    }

    #[test]
    fn str2col_base_paren() {
        assert_eq!(
            str2col("(0.2 0.5 0.6)"),
            Some((Space::SRGB, [0.2, 0.5, 0.6]))
        )
    }

    #[test]
    fn str2col_base_paren2() {
        assert_eq!(
            str2col("{ 0.2 : 0.5 : 0.6 }"),
            Some((Space::SRGB, [0.2, 0.5, 0.6]))
        )
    }

    #[test]
    fn str2col_base_none() {
        assert_eq!(str2col("  0.2   0.5     f"), None)
    }

    #[test]
    fn str2col_base_none2() {
        assert_eq!(str2col("0.2*0.5 0.6"), None)
    }

    #[test]
    fn str2col_base_paren_none() {
        assert_eq!(str2col("(0.2 0.5 0.6"), None)
    }

    #[test]
    fn str2col_base_paren_none2() {
        assert_eq!(str2col("0.2 0.5 0.6}"), None)
    }

    #[test]
    fn str2col_lch() {
        assert_eq!(
            str2col("lch(50, 30, 160)"),
            Some((Space::CIELCH, [50.0, 30.0, 160.0]))
        )
    }

    #[test]
    fn str2col_lch_space() {
        assert_eq!(
            str2col("lch 50, 30, 160"),
            Some((Space::CIELCH, [50.0, 30.0, 160.0]))
        )
    }

    #[test]
    fn str2col_lch_colon() {
        assert_eq!(
            str2col("lch:50:30:160"),
            Some((Space::CIELCH, [50.0, 30.0, 160.0]))
        )
    }

    #[test]
    fn str2col_lch_semicolon() {
        assert_eq!(
            str2col("lch;50;30;160"),
            Some((Space::CIELCH, [50.0, 30.0, 160.0]))
        )
    }

    #[test]
    fn str2col_lch_mixed() {
        assert_eq!(
            str2col("lch; (50,30,160)"),
            Some((Space::CIELCH, [50.0, 30.0, 160.0]))
        )
    }

    #[test]
    fn str2col_lch_mixed2() {
        assert_eq!(
            str2col("lch(50; 30; 160)"),
            Some((Space::CIELCH, [50.0, 30.0, 160.0]))
        )
    }

    #[test]
    fn str2col_lch_mixed3() {
        assert_eq!(
            str2col("lch   (50   30  160)"),
            Some((Space::CIELCH, [50.0, 30.0, 160.0]))
        )
    }

    #[test]
    fn str2col_hex() {
        assert_eq!(str2col(HEX), Some((Space::SRGB, irgb_to_srgb(IRGB))))
    }

    #[test]
    fn str2space_base() {
        let pix = str2space("oklch : 0.62792590, 0.25768453, 29.22319405", Space::SRGB)
            .expect("STR2SPACE_BASE FAIL");
        let reference = [1.00000000, 0.00000000, 0.00000000];
        pix_cmp(&[pix], &[reference], 1e-3, &[]);
    }

    #[test]
    fn str2space_hex() {
        let pix = str2space(" { #FF0000 } ", Space::OKLCH).expect("STR2SPACE_HEX FAIL");
        let reference = [0.62792590, 0.25768453, 29.22319405];
        pix_cmp(&[pix], &[reference], 1e-3, &[]);
    }
    // ### Str2Col ### }}}
}
// ### TESTS ### }}}
