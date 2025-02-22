#![warn(missing_docs)]

//! Comprehensive colorspace conversions in pure Rust
//!
//! The working data structure is `[DType; ValidChannels]`, where DType is one of
//! `f32` or `f64` and ValidChannels is either 3 or 4, with the 4th channel representing
//! alpha and being unprocessed outside of typing conversions
//!
//! Formulae are generally taken from their research papers or Wikipedia and validated against
//! colour-science <https://github.com/colour-science/colour>
//!
//! This crate references CIE Standard Illuminant D65 for functions to/from CIE XYZ

#[cfg(test)]
mod tests;

mod generated_quantiles;

use core::cmp::PartialOrd;
use core::ffi::{c_char, CStr};
use core::fmt::{Debug, Display};
use core::ops::{Add, Div, Mul, Neg, Rem, Sub};

// DType {{{

/// 3 channels, or 4 with alpha.
/// Alpha ignored during space conversions.
pub struct Channels<const N: usize>;
/// 3 channels, or 4 with alpha.
/// Alpha ignored during space conversions.
pub trait ValidChannels {}
impl ValidChannels for Channels<3> {}
impl ValidChannels for Channels<4> {}

#[allow(missing_docs)]
/// Convert an F32 ito any supported DType
pub trait FromF32: Sized {
    fn ff32(f: f32) -> Self;
}

impl FromF32 for f32 {
    fn ff32(f: f32) -> Self {
        f
    }
}

impl FromF32 for f64 {
    fn ff32(f: f32) -> Self {
        f.into()
    }
}

trait ToDType<T>: Sized {
    fn to_dt(self) -> T;
}

impl<U> ToDType<U> for f32
where
    U: FromF32 + Sized,
{
    fn to_dt(self) -> U {
        FromF32::ff32(self)
    }
}

#[allow(missing_docs)]
/// Trait for all supported data types in colcon
pub trait DType:
    Sized
    + Copy
    + Add<Output = Self>
    + Div<Output = Self>
    + Mul<Output = Self>
    + Neg<Output = Self>
    + Rem<Output = Self>
    + Sub<Output = Self>
    + PartialOrd
    + Debug
    + Display
    + FromF32
{
    fn powi(self, rhs: i32) -> Self;
    fn powf(self, rhs: Self) -> Self;
    /// Sign-agnostic powf
    fn spowf(self, rhs: Self) -> Self;
    fn rem_euclid(self, rhs: Self) -> Self;

    fn abs(self) -> Self;
    fn trunc(self) -> Self;
    fn max(self, other: Self) -> Self;
    fn min(self, other: Self) -> Self;

    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn to_degrees(self) -> Self;
    fn to_radians(self) -> Self;
    fn atan2(self, rhs: Self) -> Self;

    fn sqrt(self) -> Self {
        self.powf((1.0 / 2.0).to_dt())
    }
    fn cbrt(self) -> Self {
        self.powf((1.0 / 3.0).to_dt())
    }
    fn ssqrt(self) -> Self {
        self.spowf((1.0 / 2.0).to_dt())
    }
    fn scbrt(self) -> Self {
        self.spowf((1.0 / 3.0).to_dt())
    }

    fn _fma(self, mul: Self, add: Self) -> Self;
    /// Fused multiply-add if "fma" is enabled in rustc
    fn fma(self, mul: Self, add: Self) -> Self {
        // other non-x86 names?
        if cfg!(target_feature = "fma") {
            self._fma(mul, add) // crazy slow without FMA3
        } else {
            self * mul + add
        }
    }
}

macro_rules! impl_float {
    ($type:ident) => {
        impl DType for $type {
            fn powi(self, rhs: i32) -> Self {
                self.powi(rhs)
            }
            fn powf(self, rhs: Self) -> Self {
                self.powf(rhs)
            }
            fn spowf(self, rhs: Self) -> Self {
                self.abs().powf(rhs).copysign(self)
            }
            fn rem_euclid(self, rhs: Self) -> Self {
                self.rem_euclid(rhs)
            }
            fn abs(self) -> Self {
                self.abs()
            }
            fn trunc(self) -> Self {
                self.trunc()
            }
            fn max(self, other: Self) -> Self {
                self.max(other)
            }
            fn min(self, other: Self) -> Self {
                self.min(other)
            }
            fn sin(self) -> Self {
                self.sin()
            }
            fn cos(self) -> Self {
                self.cos()
            }
            fn to_degrees(self) -> Self {
                self.to_degrees()
            }
            fn to_radians(self) -> Self {
                self.to_radians()
            }
            fn atan2(self, rhs: Self) -> Self {
                self.atan2(rhs)
            }
            fn sqrt(self) -> Self {
                self.sqrt()
            }
            // 50% slower than powf/spowf?
            //fn cbrt(self) -> Self {
            //    self.cbrt()
            //}
            fn _fma(self, mul: Self, add: Self) -> Self {
                self.mul_add(mul, add)
            }
        }
    };
}

impl_float!(f32);
impl_float!(f64);

// }}}

/// Create an array of separate channel buffers from a single interwoven buffer.
/// Copies the data.
pub fn unweave<T, const N: usize>(slice: &[T]) -> [Box<[T]>; N]
where
    T: Debug + Copy,
{
    let len = slice.len() / N;
    let mut result: [Vec<T>; N] = (0..N)
        .map(|_| Vec::with_capacity(len))
        .collect::<Vec<Vec<T>>>()
        .try_into()
        .unwrap();

    slice.chunks_exact(N).for_each(|chunk| {
        chunk.iter().zip(result.iter_mut()).for_each(|(v, arr)| arr.push(*v));
    });

    result.map(|v| v.into_boxed_slice())
}

/// Create a monolithic woven buffer using unwoven independent channel buffers.
/// Copies the data.
pub fn weave<T, const N: usize>(array: [Box<[T]>; N]) -> Box<[T]>
where
    T: Debug + Copy,
{
    let len = array[0].len();
    (0..len)
        .into_iter()
        .fold(Vec::with_capacity(len * N), |mut acc, it| {
            (0..N).into_iter().for_each(|n| acc.push(array[n][it]));
            acc
        })
        .into_boxed_slice()
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
const PQEOTF_M1: f32 = 2610. / 16384.;
const PQEOTF_M2: f32 = 2523. / 4096. * 128.;
const PQEOTF_C1: f32 = 3424. / 4096.;
const PQEOTF_C2: f32 = 2413. / 4096. * 32.;
const PQEOTF_C3: f32 = 2392. / 4096. * 32.;

// JzAzBz
const JZAZBZ_B: f32 = 1.15;
const JZAZBZ_G: f32 = 0.66;
const JZAZBZ_D: f32 = -0.56;
const JZAZBZ_D0: f32 = 1.6295499532821566e-11;
const JZAZBZ_P: f32 = 1.7 * PQEOTF_M2;

// ### CONSTS ### }}}

// ### MATRICES ### {{{

/// Const matrix upcast.
/// What I wouldn't give for .map() to work
const fn m32_to_m64(m: [[f32; 3]; 3]) -> [[f64; 3]; 3] {
    [
        [m[0][0] as f64, m[0][1] as f64, m[0][2] as f64],
        [m[1][0] as f64, m[1][1] as f64, m[1][2] as f64],
        [m[2][0] as f64, m[2][1] as f64, m[2][2] as f64],
    ]
}

/// Matrix determinant using only constant math
const fn det(m: [[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// Matrix inversion using only constant math
/// Panics if determinant is zero
/// Compute scale of f64
const fn inv(m: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let m = m32_to_m64(m);
    let d = det(m);
    if d == 0.0 {
        panic!("Matrix is singular and has no inverse")
    }

    [
        [
            ((m[1][1] * m[2][2] - m[1][2] * m[2][1]) / d) as f32,
            ((m[0][2] * m[2][1] - m[0][1] * m[2][2]) / d) as f32,
            ((m[0][1] * m[1][2] - m[0][2] * m[1][1]) / d) as f32,
        ],
        [
            ((m[1][2] * m[2][0] - m[1][0] * m[2][2]) / d) as f32,
            ((m[0][0] * m[2][2] - m[0][2] * m[2][0]) / d) as f32,
            ((m[0][2] * m[1][0] - m[0][0] * m[1][2]) / d) as f32,
        ],
        [
            ((m[1][0] * m[2][1] - m[1][1] * m[2][0]) / d) as f32,
            ((m[0][1] * m[2][0] - m[0][0] * m[2][1]) / d) as f32,
            ((m[0][0] * m[1][1] - m[0][1] * m[1][0]) / d) as f32,
        ],
    ]
}

/// Its easier to write matricies visually then transpose them so they can be indexed per vector
/// [X1, X2] -> [X1, Y1]
/// [Y1, Y2]    [X2, Y2]
const fn t(m: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

/// Matrix Multiply
fn mm<T: DType>(m: [[f32; 3]; 3], p: [T; 3]) -> [T; 3] {
    [
        p[0].fma(m[0][0].to_dt(), p[1].fma(m[1][0].to_dt(), p[2] * m[2][0].to_dt())),
        p[0].fma(m[0][1].to_dt(), p[1].fma(m[1][1].to_dt(), p[2] * m[2][1].to_dt())),
        p[0].fma(m[0][2].to_dt(), p[1].fma(m[1][2].to_dt(), p[2] * m[2][2].to_dt())),
    ]
}

// CIE XYZ
const XYZ65_MAT: [[f32; 3]; 3] = t([
    [0.4124, 0.3576, 0.1805],
    [0.2126, 0.7152, 0.0722],
    [0.0193, 0.1192, 0.9505],
]);

// OKLAB
// They appear to be provided already transposed for code in the blog post
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

// JzAzBz
const JZAZBZ_M1: [[f32; 3]; 3] = t([
    [0.41478972, 0.579999, 0.0146480],
    [-0.2015100, 1.120649, 0.0531008],
    [-0.0166008, 0.264800, 0.6684799],
]);
const JZAZBZ_M2: [[f32; 3]; 3] = t([
    [0.500000, 0.500000, 0.000000],
    [3.524000, -4.066708, 0.542708],
    [0.199076, 1.096799, -1.295875],
]);

// ICtCp
const ICTCP_M1: [[f32; 3]; 3] = t([
    [1688. / 4096., 2146. / 4096., 262. / 4096.],
    [683. / 4096., 2951. / 4096., 462. / 4096.],
    [99. / 4096., 309. / 4096., 3688. / 4096.],
]);
const ICTCP_M2: [[f32; 3]; 3] = t([
    [2048. / 4096., 2048. / 4096., 0. / 4096.],
    [6610. / 4096., -13613. / 4096., 7003. / 4096.],
    [17933. / 4096., -17390. / 4096., -543. / 4096.],
]);

// ### MATRICES ### }}}

// ### TRANSFER FUNCTIONS ### {{{

/// sRGB Electro-Optical Transfer Function
///
/// <https://en.wikipedia.org/wiki/SRGB#Computing_the_transfer_function>
pub fn srgb_eotf<T: DType>(n: T) -> T {
    if n <= SRGBEOTF_CHI.to_dt() {
        n / SRGBEOTF_PHI.to_dt()
    } else {
        ((n + SRGBEOTF_ALPHA.to_dt()) / (SRGBEOTF_ALPHA + 1.0).to_dt()).powf(SRGBEOTF_GAMMA.to_dt())
    }
}

/// Inverse sRGB Electro-Optical Transfer Function
///
/// <https://en.wikipedia.org/wiki/SRGB#Computing_the_transfer_function>
pub fn srgb_oetf<T: DType>(n: T) -> T {
    if n <= SRGBEOTF_CHI_INV.to_dt() {
        n * SRGBEOTF_PHI.to_dt()
    } else {
        (n.powf((1.0 / SRGBEOTF_GAMMA).to_dt())).fma((1.0 + SRGBEOTF_ALPHA).to_dt(), (-SRGBEOTF_ALPHA).to_dt())
    }
}

// <https://www.itu.int/rec/R-REC-BT.2100/en> Table 4 "Reference PQ EOTF"
fn pq_eotf_common<T: DType>(e: T, m2: T) -> T {
    let ep_pow_1divm2 = e.spowf(T::ff32(1.0) / m2);

    let numerator: T = (ep_pow_1divm2 - PQEOTF_C1.to_dt()).max(0.0.to_dt());
    let denominator: T = ep_pow_1divm2.fma(T::ff32(-PQEOTF_C3), PQEOTF_C2.to_dt());

    let y = (numerator / denominator).spowf((1.0 / PQEOTF_M1).to_dt());

    y * 10000.0.to_dt()
}

// <https://www.itu.int/rec/R-REC-BT.2100/en> Table 4 "Reference PQ OETF"
fn pq_oetf_common<T: DType>(f: T, m2: T) -> T {
    let y = f / 10000.0.to_dt();
    let y_pow_m1 = y.spowf(PQEOTF_M1.to_dt());

    let numerator: T = T::ff32(PQEOTF_C2).fma(y_pow_m1, PQEOTF_C1.to_dt());
    let denominator: T = T::ff32(PQEOTF_C3).fma(y_pow_m1, 1.0.to_dt());

    (numerator / denominator).spowf(m2)
}

/// Dolby Perceptual Quantizer Electro-Optical Transfer Function primarily used for ICtCP
///
/// <https://www.itu.int/rec/R-REC-BT.2100/en> Table 4 "Reference PQ EOTF"
pub fn pq_eotf<T: DType>(e: T) -> T {
    pq_eotf_common(e, PQEOTF_M2.to_dt())
}

/// Dolby Perceptual Quantizer Optical-Electro Transfer Function primarily used for ICtCP
///
/// <https://www.itu.int/rec/R-REC-BT.2100/en> Table 4 "Reference PQ OETF"
pub fn pq_oetf<T: DType>(f: T) -> T {
    pq_oetf_common(f, PQEOTF_M2.to_dt())
}

/// Dolby Perceptual Quantizer Electro-Optical Transfer Function modified for JzAzBz
///
/// Replaced PQEOTF_M2 with JZAZBZ_P
///
/// <https://www.itu.int/rec/R-REC-BT.2100/en> Table 4 "Reference PQ EOTF"
pub fn pqz_eotf<T: DType>(e: T) -> T {
    pq_eotf_common(e, JZAZBZ_P.to_dt())
}

/// Dolby Perceptual Quantizer Optical-Electro Transfer Function modified for JzAzBz
///
/// Replaced PQEOTF_M2 with JZAZBZ_P
///
/// <https://www.itu.int/rec/R-REC-BT.2100/en> Table 4 "Reference PQ OETF"
pub fn pqz_oetf<T: DType>(f: T) -> T {
    pq_oetf_common(f, JZAZBZ_P.to_dt())
}

// ### TRANSFER FUNCTIONS ### }}}

// ### Helmholtz-Kohlrausch ### {{{

/// Extended K-values from High et al 2021/2022
const K_HIGH2022: [f32; 4] = [0.1644, 0.0603, 0.1307, 0.0060];

/// Mean value of the HK delta for CIE LCH(ab), High et al 2023 implementation.
///
/// Measured with 36000 steps in the hk_exmample file @ 100 C(ab).
/// Cannot make a const fn: <https://github.com/rust-lang/rust/issues/57241>
pub const HIGH2023_MEAN: f32 = 20.956442;

/// Returns difference in perceptual lightness based on hue, aka the Helmholtz-Kohlrausch effect.
/// High et al 2023 implementation.
pub fn hk_high2023<T: DType, const N: usize>(lch: &[T; N]) -> T
where
    Channels<N>: ValidChannels,
{
    let fby: T = T::ff32(K_HIGH2022[0]).fma(
        ((lch[2] - 90.0.to_dt()) / 2.0.to_dt()).to_radians().sin().abs(),
        K_HIGH2022[1].to_dt(),
    );

    let fr: T = if lch[2] <= 90.0.to_dt() || lch[2] >= 270.0.to_dt() {
        T::ff32(K_HIGH2022[2]).fma(lch[2].to_radians().cos().abs(), K_HIGH2022[3].to_dt())
    } else {
        0.0.to_dt()
    };

    (fby + fr) * lch[1]
}

/// Compensates CIE LCH's L value for the Helmholtz-Kohlrausch effect.
/// High et al 2023 implementation.
pub fn hk_high2023_comp<T: DType, const N: usize>(lch: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    lch[0] = lch[0] + (T::ff32(HIGH2023_MEAN) - hk_high2023(lch)) * (lch[1] / 100.0.to_dt())
}

// ### Helmholtz-Kohlrausch ### }}}

// ### Space ### {{{

/// Defines colorspace pixels will take.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Space {
    /// Gamma-corrected sRGB.
    SRGB,

    /// Hue Saturation Value.
    ///
    /// A UCS typically preferred for modern applications
    HSV,

    /// Linear RGB. IEC 61966-2-1:1999 transferred
    LRGB,

    /// 1931 CIE XYZ @ D65.
    XYZ,

    /// CIE LAB. Lightness, red/green chromacity, yellow/blue chromacity.
    ///
    /// 1976 UCS with many known flaws. Most other LAB spaces derive from this
    CIELAB,

    /// CIE LCH(ab). Lightness, Chroma, Hue
    ///
    /// Cylindrical version of CIE LAB.
    CIELCH,

    /// Oklab
    ///
    /// <https://bottosson.github.io/posts/oklab/>
    ///
    /// 2020 UCS, used in CSS Color Module Level 4
    OKLAB,

    /// Cylindrical version of OKLAB.
    OKLCH,

    /// JzAzBz
    ///
    /// <https://opg.optica.org/oe/fulltext.cfm?uri=oe-25-13-15131>
    ///
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

impl TryFrom<*const c_char> for Space {
    type Error = ();
    fn try_from(value: *const c_char) -> Result<Self, ()> {
        if value.is_null() {
            Err(())
        } else {
            unsafe { CStr::from_ptr(value) }
                .to_str()
                .ok()
                .map(|s| Self::try_from(s).ok())
                .flatten()
                .ok_or(())
        }
    }
}

impl Display for Space {
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

    /// Retrieves a map from a given Space back to SRGB.
    ///
    /// This is useful for things like creating adjustable values in Space
    /// that represent most of the SRGB range without clipping.
    /// Wrapping Hue values are set to f32::INFINITY
    pub const fn srgb_quants(&self) -> [[f32; 3]; 101] {
        //[[0.0; 3]; 101]
        generated_quantiles::srgb_quants(self)
    }
}

// ### Space ### }}}

// ### Convert Space ### {{{

macro_rules! op_single {
    ($func:ident, $data:expr) => {
        $func($data)
    };
}

macro_rules! op_chunk {
    ($func:ident, $data:expr) => {
        $data.iter_mut().for_each(|pixel| $func(pixel))
    };
}

#[rustfmt::skip]
macro_rules! graph {
    ($recurse:ident, $data:expr, $from:expr, $to:expr, $op:ident) => {
        match ($from, $to) {
            // no-ops
            (Space::HSV, Space::HSV) => (),
            (Space::SRGB, Space::SRGB) => (),
            (Space::LRGB, Space::LRGB) => (),
            (Space::XYZ, Space::XYZ) => (),
            (Space::CIELAB, Space::CIELAB) => (),
            (Space::CIELCH, Space::CIELCH) => (),
            (Space::OKLAB, Space::OKLAB) => (),
            (Space::OKLCH, Space::OKLCH) => (),
            (Space::JZAZBZ, Space::JZAZBZ) => (),
            (Space::JZCZHZ, Space::JZCZHZ) => (),

            //endcaps
            (Space::SRGB, Space::HSV) => $op!(srgb_to_hsv, $data),
            (Space::CIELAB, Space::CIELCH)
            | (Space::OKLAB, Space::OKLCH)
            | (Space::JZAZBZ, Space::JZCZHZ) => $op!(lab_to_lch, $data),

            // Reverse Endcaps
            (Space::HSV, _) => { $op!(hsv_to_srgb, $data); $recurse(Space::SRGB, $to, $data) }
            (Space::CIELCH, _) => { $op!(lch_to_lab, $data); $recurse(Space::CIELAB, $to, $data) }
            (Space::OKLCH, _) => { $op!(lch_to_lab, $data); $recurse(Space::OKLAB, $to, $data) }
            (Space::JZCZHZ, _) => { $op!(lch_to_lab, $data); $recurse(Space::JZAZBZ, $to, $data) }

            // SRGB Up
            (Space::SRGB, _) => { $op!(srgb_to_lrgb, $data); $recurse(Space::LRGB, $to, $data) }

            // LRGB Down
            (Space::LRGB, Space::SRGB | Space::HSV) => { $op!(lrgb_to_srgb, $data); $recurse(Space::SRGB, $to, $data) }
            // LRGB Up
            (Space::LRGB, _) => { $op!(lrgb_to_xyz, $data); $recurse(Space::XYZ, $to, $data) }

            // XYZ Down
            (Space::XYZ, Space::SRGB | Space::LRGB | Space::HSV) => { $op!(xyz_to_lrgb, $data); $recurse(Space::LRGB, $to, $data) }
            // XYZ Up
            (Space::XYZ, Space::CIELAB | Space::CIELCH) => { $op!(xyz_to_cielab, $data); $recurse(Space::CIELAB, $to, $data) }
            (Space::XYZ, Space::OKLAB | Space::OKLCH) => { $op!(xyz_to_oklab, $data); $recurse(Space::OKLAB, $to, $data) }
            (Space::XYZ, Space::JZAZBZ | Space::JZCZHZ) => { $op!(xyz_to_jzazbz, $data); $recurse(Space::JZAZBZ, $to, $data) }

            // LAB Down
            (Space::CIELAB, _) => { $op!(cielab_to_xyz, $data); $recurse(Space::XYZ, $to, $data) }
            (Space::OKLAB, _) => { $op!(oklab_to_xyz, $data); $recurse(Space::XYZ, $to, $data) }
            (Space::JZAZBZ, _) => { $op!(jzazbz_to_xyz, $data); $recurse(Space::XYZ, $to, $data) }
        }
    };
}

/// Runs conversion functions to convert `pixel` from one `Space` to another
/// in the least possible moves.
pub fn convert_space<T: DType, const N: usize>(from: Space, to: Space, pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    graph!(convert_space, pixel, from, to, op_single);
}

/// Runs conversion functions to convert `pixel` from one `Space` to another
/// in the least possible moves.
///
/// Caches conversion graph for faster iteration.
pub fn convert_space_chunked<T: DType, const N: usize>(from: Space, to: Space, pixels: &mut [[T; N]])
where
    Channels<N>: ValidChannels,
{
    graph!(convert_space_chunked, pixels, from, to, op_chunk);
}

/// Runs conversion functions to convert `pixel` from one `Space` to another
/// in the least possible moves.
///
/// Caches conversion graph for faster iteration and ignores remainder values in slice.
pub fn convert_space_sliced<T: DType, const N: usize>(from: Space, to: Space, pixels: &mut [T])
where
    Channels<N>: ValidChannels,
{
    // Inline std::slice::as_chunks_mut without the asserts as its already guarded by ValidChannels
    let (mut_chunks, _remainder): (&mut [[T; N]], &mut [T]) = {
        let len = pixels.len() / N;
        let (multiple_of_n, remainder) = pixels.split_at_mut(len * N);
        let array_slice = {
            let this = &mut *multiple_of_n;
            let new_len = this.len() / N;
            unsafe { core::slice::from_raw_parts_mut(this.as_mut_ptr().cast(), new_len) }
        };
        (array_slice, remainder)
    };
    graph!(convert_space_chunked, mut_chunks, from, to, op_chunk);
}

/// Same as `convert_space_sliced` but with FFI types.
///
/// Returns 0 on success, 1 on invalid `from`, 2 on invalid `to`, 3 on invalid `pixels`
///
/// `len` is in elements rather than bytes
pub fn convert_space_ffi<T: DType, const N: usize>(
    from: *const c_char,
    to: *const c_char,
    pixels: *mut T,
    len: usize,
) -> i32
where
    Channels<N>: ValidChannels,
{
    let Ok(from) = Space::try_from(from) else { return 1 };
    let Ok(to) = Space::try_from(to) else { return 2 };
    let pixels = unsafe {
        if pixels.is_null() {
            return 3;
        } else {
            core::slice::from_raw_parts_mut(pixels, len)
        }
    };
    convert_space_sliced::<T, N>(from, to, pixels);
    0
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
/// Separated with spaces, ';', ':', or ','
///
/// Can additionally be set as a % of SDR range.
///
/// Alpha will be NaN if only 3 values are provided.
///
/// # Examples
///
/// ```
/// use colcon::{str2col, Space};
///
/// assert_eq!(str2col("0.2, 0.5, 0.6"), Some((Space::SRGB, [0.2f32, 0.5, 0.6])));
/// assert_eq!(str2col("lch:50;20;120"), Some((Space::CIELCH, [50.0f32, 20.0, 120.0])));
/// assert_eq!(str2col("oklab(0.2, 0.6, -0.5)"), Some((Space::OKLAB, [0.2f32, 0.6, -0.5])));
/// assert_eq!(str2col("srgb 100% 50% 25%"), Some((Space::SRGB, [1.0f32, 0.5, 0.25])));
/// ```
pub fn str2col<T: DType, const N: usize>(mut s: &str) -> Option<(Space, [T; N])>
where
    Channels<N>: ValidChannels,
{
    s = rm_paren(s.trim());
    let mut space = Space::SRGB;
    let mut result = [f32::NAN; N];

    // Return hex if valid
    if let Ok(irgb) = hex_to_irgb(s) {
        return Some((space, irgb_to_srgb(irgb)));
    }

    let seps = [',', ':', ';'];

    // Find Space at front then trim
    if let Some(i) = s.find(|c: char| c.is_whitespace() || seps.contains(&c) || ['(', '[', '{'].contains(&c)) {
        if let Ok(sp) = Space::try_from(&s[..i]) {
            space = sp;
            s = rm_paren(s[i..].trim_start_matches(|c: char| c.is_whitespace() || seps.contains(&c)));
        }
    }

    // Split by separators + whitespace and parse
    for (n, split) in s
        .split(|c: char| c.is_whitespace() || seps.contains(&c))
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        if n > 3 {
            return None;
        } else if n >= result.len() {
            continue;
        } else if let Ok(value) = split.parse::<f32>() {
            result[n] = value;
        } else if split.ends_with('%') {
            if let Ok(percent) = split[0..(split.len() - 1)].parse::<f32>() {
                // alpha
                if n == 3 {
                    result[n] = percent / 100.0;
                    continue;
                }
                let (q0, q100) = (space.srgb_quants()[0][n], space.srgb_quants()[100][n]);
                if q0.is_finite() && q100.is_finite() {
                    result[n] = percent / 100.0 * (q100 - q0) + q0;
                } else if Space::UCS_POLAR.contains(&space) {
                    result[n] = percent / 100.0 * 360.0
                } else if space == Space::HSV {
                    result[n] = percent / 100.0
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    if result.iter().take(3).all(|v| v.is_finite()) {
        Some((space, result.map(|c| c.to_dt())))
    } else {
        None
    }
}

/// Convert a string into a pixel of the requested Space.
///
/// Shorthand for str2col() -> convert_space()
pub fn str2space<T: DType, const N: usize>(s: &str, to: Space) -> Option<[T; N]>
where
    Channels<N>: ValidChannels,
{
    str2col(s).map(|(from, mut col)| {
        convert_space(from, to, &mut col);
        col
    })
}

/// Same as `str2space` but with FFI types
///
/// Returns an N-length pointer to T on success or null on failure
pub fn str2space_ffi<T: DType, const N: usize>(s: *const c_char, to: *const c_char) -> *const T
where
    Channels<N>: ValidChannels,
{
    if s.is_null() {
        return core::ptr::null();
    };
    let Some(s) = unsafe { CStr::from_ptr(s) }.to_str().ok() else {
        return core::ptr::null();
    };
    let Ok(to) = Space::try_from(to) else {
        return core::ptr::null();
    };
    str2space::<T, N>(s, to).map_or(core::ptr::null(), |b| Box::into_raw(Box::new(b)).cast())
}
// ### Str2Col ### }}}

// ### FORWARD ### {{{

/// Convert floating (0.0..1.0) RGB to integer (0..255) RGB.
pub fn srgb_to_irgb<const N: usize>(pixel: [f32; N]) -> [u8; N]
where
    Channels<N>: ValidChannels,
{
    pixel.map(|c| ((c * 255.0).round().max(0.0).min(255.0) as u8))
}

/// Create a hexadecimal string from integer RGB.
pub fn irgb_to_hex<const N: usize>(pixel: [u8; N]) -> String
where
    Channels<N>: ValidChannels,
{
    let mut hex = String::with_capacity(N * 2 + 1);
    hex.push('#');

    pixel.into_iter().for_each(|c| {
        [c / 16, c % 16]
            .into_iter()
            .for_each(|n| hex.push(if n >= 10 { n + 55 } else { n + 48 } as char))
    });

    hex
}

/// Convert from sRGB to HSV.
pub fn srgb_to_hsv<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    let vmin = pixel[0].min(pixel[1]).min(pixel[2]);
    let vmax = pixel[0].max(pixel[1]).max(pixel[2]);
    let dmax = vmax - vmin;

    let v = vmax;

    let (h, s): (T, T) = if dmax == 0.0.to_dt() {
        (0.0.to_dt(), 0.0.to_dt())
    } else {
        let s = dmax / vmax;

        let [branch_0, branch_1] = [pixel[0] == vmax, pixel[1] == vmax];

        pixel.iter_mut().take(3).for_each(|c| {
            *c = (((vmax - *c) / 6.0.to_dt()) + (dmax / 2.0.to_dt())) / dmax;
        });

        let h = if branch_0 {
            pixel[2] - pixel[1]
        } else if branch_1 {
            T::ff32(1.0 / 3.0) + pixel[0] - pixel[2]
        } else {
            T::ff32(2.0 / 3.0) + pixel[1] - pixel[0]
        }
        .rem_euclid(1.0.to_dt());
        (h, s)
    };
    pixel[0] = h;
    pixel[1] = s;
    pixel[2] = v;
}

/// Convert from sRGB to Linear RGB by applying the sRGB EOTF
///
/// <https://www.color.org/chardata/rgb/srgb.xalter>
pub fn srgb_to_lrgb<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    pixel.iter_mut().take(3).for_each(|c| *c = srgb_eotf(*c));
}

/// Convert from Linear Light RGB to CIE XYZ, D65 standard illuminant
///
/// <https://en.wikipedia.org/wiki/SRGB#From_sRGB_to_CIE_XYZ>
pub fn lrgb_to_xyz<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    [pixel[0], pixel[1], pixel[2]] = mm(XYZ65_MAT, [pixel[0], pixel[1], pixel[2]])
}

/// Convert from CIE XYZ to CIE LAB.
///
/// <https://en.wikipedia.org/wiki/CIELAB_color_space#From_CIEXYZ_to_CIELAB>
pub fn xyz_to_cielab<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    // Reverse D65 standard illuminant
    pixel.iter_mut().take(3).zip(D65).for_each(|(c, d)| *c = *c / d.to_dt());

    pixel.iter_mut().take(3).for_each(|c| {
        if *c > T::ff32(LAB_DELTA).powi(3) {
            *c = c.cbrt()
        } else {
            *c = *c / (3.0 * LAB_DELTA.powi(2)).to_dt() + (4f32 / 29f32).to_dt()
        }
    });

    [pixel[0], pixel[1], pixel[2]] = [
        T::ff32(116.0).fma(pixel[1], T::ff32(-16.0)),
        T::ff32(500.0) * (pixel[0] - pixel[1]),
        T::ff32(200.0) * (pixel[1] - pixel[2]),
    ]
}

/// Convert from CIE XYZ to OKLAB.
///
/// <https://bottosson.github.io/posts/oklab/>
pub fn xyz_to_oklab<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    let mut lms = mm(OKLAB_M1, [pixel[0], pixel[1], pixel[2]]);
    lms.iter_mut().for_each(|c| *c = c.scbrt());
    [pixel[0], pixel[1], pixel[2]] = mm(OKLAB_M2, lms);
}

/// Convert CIE XYZ to JzAzBz
///
/// <https://opg.optica.org/oe/fulltext.cfm?uri=oe-25-13-15131>
pub fn xyz_to_jzazbz<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    let mut lms = mm(
        JZAZBZ_M1,
        [
            pixel[0].fma(JZAZBZ_B.to_dt(), T::ff32(-JZAZBZ_B + 1.0) * pixel[2]),
            pixel[1].fma(JZAZBZ_G.to_dt(), T::ff32(-JZAZBZ_G + 1.0) * pixel[0]),
            pixel[2],
        ],
    );

    lms.iter_mut().for_each(|e| *e = pqz_oetf(*e));

    let lab = mm(JZAZBZ_M2, lms);

    pixel[0] = (T::ff32(1.0 + JZAZBZ_D) * lab[0]) / lab[0].fma(JZAZBZ_D.to_dt(), 1.0.to_dt()) - JZAZBZ_D0.to_dt();
    pixel[1] = lab[1];
    pixel[2] = lab[2];
}

// Disabled for now as all the papers are paywalled
// /// Convert CIE XYZ to CAM16-UCS
// #[unsafe(no_mangle)]
// pub extern "C" fn xyz_to_cam16ucs(pixel: &mut [f32; 3]) {

// }

/// Convert LRGB to ICtCp. Unvalidated, WIP
///
/// <https://www.itu.int/rec/R-REC-BT.2100/en>
pub fn _lrgb_to_ictcp<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    // <https://www.itu.int/rec/R-REC-BT.2020/en>
    // let alpha = 1.09929682680944;
    // let beta = 0.018053968510807;
    // let bt2020 = |e: &mut f32| {
    //     *e = if *e < beta {4.5 * *e}
    //     else {alpha * e.powf(0.45) - (alpha - 1.0)}
    // };
    // pixel.iter_mut().for_each(|c| bt2020(c));

    let mut lms = mm(ICTCP_M1, [pixel[0], pixel[1], pixel[2]]);
    // lms prime
    lms.iter_mut().for_each(|c| *c = pq_oetf(*c));
    [pixel[0], pixel[1], pixel[2]] = mm(ICTCP_M2, lms);
}

/// Converts an LAB based space to a cylindrical representation.
///
/// <https://en.wikipedia.org/wiki/CIELAB_color_space#Cylindrical_model>
pub fn lab_to_lch<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    [pixel[0], pixel[1], pixel[2]] = [
        pixel[0],
        (pixel[1].powi(2) + pixel[2].powi(2)).sqrt(),
        pixel[2].atan2(pixel[1]).to_degrees().rem_euclid(360.0.to_dt()),
    ];
}

// ### FORWARD ### }}}

// ### BACKWARD ### {{{

/// Convert integer (0..255) RGB to floating (0.0..1.0) RGB.
pub fn irgb_to_srgb<T: DType, const N: usize>(pixel: [u8; N]) -> [T; N]
where
    Channels<N>: ValidChannels,
{
    pixel.map(|c| T::ff32(c as f32 / 255.0))
}

/// Create integer RGB set from hex string.
/// `DEFAULT` is only used when 4 channels are requested but 3 is given.
pub fn hex_to_irgb_default<const N: usize, const DEFAULT: u8>(hex: &str) -> Result<[u8; N], String>
where
    Channels<N>: ValidChannels,
{
    let mut chars = hex.trim().chars();
    if chars.as_str().starts_with('#') {
        chars.next();
    }

    let ids: Vec<u32> = match chars.as_str().len() {
        6 | 8 => chars
            .map(|c| {
                let u = c as u32;
                // numeric
                if 57 >= u && u >= 48 {
                    Ok(u - 48)
                // uppercase
                } else if 70 >= u && u >= 65 {
                    Ok(u - 55)
                // lowercase
                } else if 102 >= u && u >= 97 {
                    Ok(u - 87)
                } else {
                    Err(String::from("Hex character '") + &String::from(c) + "' out of bounds")
                }
            })
            .collect(),
        n => Err(String::from("Incorrect hex length ") + &n.to_string()),
    }?;

    let mut result = [DEFAULT; N];

    ids.chunks(2)
        .take(result.len())
        .enumerate()
        .for_each(|(n, chunk)| result[n] = ((chunk[0]) * 16 + chunk[1]) as u8);

    Ok(result)
}

/// Create integer RGB set from hex string.
/// Will default to 255 for alpha if 4 channels requested but hex length is 6.
/// Use `hex_to_irgb_default` to customize this.
pub fn hex_to_irgb<const N: usize>(hex: &str) -> Result<[u8; N], String>
where
    Channels<N>: ValidChannels,
{
    hex_to_irgb_default::<N, 255>(hex)
}

/// Convert from HSV to sRGB.
pub fn hsv_to_srgb<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    if pixel[1] == 0.0.to_dt() {
        [pixel[0], pixel[1]] = [pixel[2]; 2];
    } else {
        let mut var_h = pixel[0] * 6.0.to_dt();
        if var_h == 6.0.to_dt() {
            var_h = 0.0.to_dt()
        }
        let var_i = var_h.trunc();
        let var_1 = pixel[2] * (T::ff32(1.0) - pixel[1]);
        let var_2 = pixel[2] * (-var_h + var_i).fma(pixel[1], 1.0.to_dt());
        let var_3 = pixel[2] * (T::ff32(-1.0) + (var_h - var_i)).fma(pixel[1], T::ff32(1.0));

        [pixel[0], pixel[1], pixel[2]] = if var_i == 0.0.to_dt() {
            [pixel[2], var_3, var_1]
        } else if var_i == 1.0.to_dt() {
            [var_2, pixel[2], var_1]
        } else if var_i == 2.0.to_dt() {
            [var_1, pixel[2], var_3]
        } else if var_i == 3.0.to_dt() {
            [var_1, var_2, pixel[2]]
        } else if var_i == 4.0.to_dt() {
            [var_3, var_1, pixel[2]]
        } else {
            [pixel[2], var_1, var_2]
        }
    }
}

/// Convert from Linear RGB to sRGB by applying the inverse sRGB EOTF
///
/// <https://www.color.org/chardata/rgb/srgb.xalter>
pub fn lrgb_to_srgb<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    pixel.iter_mut().take(3).for_each(|c| *c = srgb_oetf(*c));
}

/// Convert from CIE XYZ to Linear Light RGB.
///
/// <https://en.wikipedia.org/wiki/SRGB#From_CIE_XYZ_to_sRGB>
pub fn xyz_to_lrgb<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    [pixel[0], pixel[1], pixel[2]] = mm(inv(XYZ65_MAT), [pixel[0], pixel[1], pixel[2]])
}

/// Convert from CIE LAB to CIE XYZ.
///
/// <https://en.wikipedia.org/wiki/CIELAB_color_space#From_CIELAB_to_CIEXYZ>
pub fn cielab_to_xyz<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    pixel[0] = pixel[0].fma((1.0 / 116.0).to_dt(), (16.0 / 116.0).to_dt());
    [pixel[0], pixel[1], pixel[2]] = [
        pixel[0] + pixel[1] / 500.0.to_dt(),
        pixel[0],
        pixel[0] - pixel[2] / 200.0.to_dt(),
    ];

    pixel.iter_mut().take(3).for_each(|c| {
        if *c > LAB_DELTA.to_dt() {
            *c = c.powi(3)
        } else {
            *c = T::ff32(3.0) * LAB_DELTA.powi(2).to_dt() * (*c - (4f32 / 29f32).to_dt())
        }
    });

    pixel.iter_mut().take(3).zip(D65).for_each(|(c, d)| *c = *c * d.to_dt());
}

/// Convert from OKLAB to CIE XYZ.
///
/// <https://bottosson.github.io/posts/oklab/>
pub fn oklab_to_xyz<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    let mut lms = mm(inv(OKLAB_M2), [pixel[0], pixel[1], pixel[2]]);
    lms.iter_mut().for_each(|c| *c = c.powi(3));
    [pixel[0], pixel[1], pixel[2]] = mm(inv(OKLAB_M1), lms);
}

/// Convert JzAzBz to CIE XYZ
///
/// <https://opg.optica.org/oe/fulltext.cfm?uri=oe-25-13-15131>
pub fn jzazbz_to_xyz<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    let mut lms = mm(
        inv(JZAZBZ_M2),
        [
            (pixel[0] + JZAZBZ_D0.to_dt())
                / (pixel[0] + JZAZBZ_D0.to_dt()).fma(T::ff32(-JZAZBZ_D), T::ff32(1.0 + JZAZBZ_D)),
            pixel[1],
            pixel[2],
        ],
    );

    lms.iter_mut().for_each(|c| *c = pqz_eotf(*c));

    [pixel[0], pixel[1], pixel[2]] = mm(inv(JZAZBZ_M1), lms);

    pixel[0] = pixel[2].fma((JZAZBZ_B - 1.0).to_dt(), pixel[0]) / JZAZBZ_B.to_dt();
    pixel[1] = pixel[0].fma((JZAZBZ_G - 1.0).to_dt(), pixel[1]) / JZAZBZ_G.to_dt();
}

// Disabled for now as all the papers are paywalled
// /// Convert CAM16-UCS to CIE XYZ
// #[unsafe(no_mangle)]
// pub extern "C" fn cam16ucs_to_xyz(pixel: &mut [f32; 3]) {

// }

/// Convert ICtCp to LRGB. Unvalidated, WIP
///
/// <https://www.itu.int/rec/R-REC-BT.2100/en>
// #[unsafe(no_mangle)]
pub fn _ictcp_to_lrgb<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    // lms prime
    let mut lms = mm(inv(ICTCP_M2), [pixel[0], pixel[1], pixel[2]]);
    // non-prime lms
    lms.iter_mut().for_each(|c| *c = pq_eotf(*c));
    [pixel[0], pixel[1], pixel[2]] = mm(inv(ICTCP_M1), lms);
}

/// Retrieves an LAB based space from its cylindrical representation.
///
/// <https://en.wikipedia.org/wiki/CIELAB_color_space#Cylindrical_model>
pub fn lch_to_lab<T: DType, const N: usize>(pixel: &mut [T; N])
where
    Channels<N>: ValidChannels,
{
    [pixel[0], pixel[1], pixel[2]] = [
        pixel[0],
        pixel[1] * pixel[2].to_radians().cos(),
        pixel[1] * pixel[2].to_radians().sin(),
    ]
}

// BACKWARD }}}

// ### MONOTYPED EXTERNAL FUNCTIONS ### {{{

#[unsafe(no_mangle)]
extern "C" fn convert_space_3f32(from: *const c_char, to: *const c_char, pixels: *mut f32, len: usize) -> i32 {
    convert_space_ffi::<_, 3>(from, to, pixels, len)
}
#[unsafe(no_mangle)]
extern "C" fn convert_space_4f32(from: *const c_char, to: *const c_char, pixels: *mut f32, len: usize) -> i32 {
    convert_space_ffi::<_, 4>(from, to, pixels, len)
}
#[unsafe(no_mangle)]
extern "C" fn convert_space_3f64(from: *const c_char, to: *const c_char, pixels: *mut f64, len: usize) -> i32 {
    convert_space_ffi::<_, 3>(from, to, pixels, len)
}
#[unsafe(no_mangle)]
extern "C" fn convert_space_4f64(from: *const c_char, to: *const c_char, pixels: *mut f64, len: usize) -> i32 {
    convert_space_ffi::<_, 4>(from, to, pixels, len)
}

#[unsafe(no_mangle)]
extern "C" fn str2space_3f32(s: *const c_char, to: *const c_char) -> *const f32 {
    str2space_ffi::<f32, 3>(s, to)
}
#[unsafe(no_mangle)]
extern "C" fn str2space_4f32(s: *const c_char, to: *const c_char) -> *const f32 {
    str2space_ffi::<f32, 4>(s, to)
}
#[unsafe(no_mangle)]
extern "C" fn str2space_3f64(s: *const c_char, to: *const c_char) -> *const f64 {
    str2space_ffi::<f64, 3>(s, to)
}
#[unsafe(no_mangle)]
extern "C" fn str2space_4f64(s: *const c_char, to: *const c_char) -> *const f64 {
    str2space_ffi::<f64, 4>(s, to)
}

macro_rules! cdef1 {
    ($base:ident, $f32:ident, $f64:ident) => {
        #[unsafe(no_mangle)]
        extern "C" fn $f32(value: f32) -> f32 {
            $base(value)
        }
        #[unsafe(no_mangle)]
        extern "C" fn $f64(value: f64) -> f64 {
            $base(value)
        }
    };
}

macro_rules! cdef3 {
    ($base:ident, $f32_3:ident, $f64_3:ident, $f32_4:ident, $f64_4:ident) => {
        #[unsafe(no_mangle)]
        extern "C" fn $f32_3(pixel: &mut [f32; 3]) {
            $base(pixel)
        }
        #[unsafe(no_mangle)]
        extern "C" fn $f64_3(pixel: &mut [f64; 3]) {
            $base(pixel)
        }
        #[unsafe(no_mangle)]
        extern "C" fn $f32_4(pixel: &mut [f32; 4]) {
            $base(pixel)
        }
        #[unsafe(no_mangle)]
        extern "C" fn $f64_4(pixel: &mut [f64; 4]) {
            $base(pixel)
        }
    };
}

macro_rules! cdef31 {
    ($base:ident, $f32_3:ident, $f64_3:ident, $f32_4:ident, $f64_4:ident) => {
        #[unsafe(no_mangle)]
        extern "C" fn $f32_3(pixel: &[f32; 3]) -> f32 {
            $base(pixel)
        }
        #[unsafe(no_mangle)]
        extern "C" fn $f64_3(pixel: &[f64; 3]) -> f64 {
            $base(pixel)
        }
        #[unsafe(no_mangle)]
        extern "C" fn $f32_4(pixel: &[f32; 4]) -> f32 {
            $base(pixel)
        }
        #[unsafe(no_mangle)]
        extern "C" fn $f64_4(pixel: &[f64; 4]) -> f64 {
            $base(pixel)
        }
    };
}

// Transfer Functions
cdef1!(srgb_eotf, srgb_eotf_f32, srgb_eotf_f64);
cdef1!(srgb_oetf, srgb_oetf_f32, srgb_oetf_f64);
cdef1!(pq_eotf, pq_eotf_f32, pq_eotf_f64);
cdef1!(pqz_eotf, pqz_eotf_f32, pqz_eotf_f64);
cdef1!(pq_oetf, pq_oetf_f32, pq_oetf_f64);
cdef1!(pqz_oetf, pqz_oetf_f32, pqz_oetf_f64);

// Helmholtz-Kohlrausch
cdef31!(
    hk_high2023,
    hk_high2023_3f32,
    hk_high2023_3f64,
    hk_high2023_4f32,
    hk_high2023_4f64
);
cdef3!(
    hk_high2023_comp,
    hk_high2023_comp_3f32,
    hk_high2023_comp_3f64,
    hk_high2023_comp_4f32,
    hk_high2023_comp_4f64
);

// Forward
cdef3!(
    srgb_to_hsv,
    srgb_to_hsv_3f32,
    srgb_to_hsv_3f64,
    srgb_to_hsv_4f32,
    srgb_to_hsv_4f64
);
cdef3!(
    srgb_to_lrgb,
    srgb_to_lrgb_3f32,
    srgb_to_lrgb_3f64,
    srgb_to_lrgb_4f32,
    srgb_to_lrgb_4f64
);
cdef3!(
    lrgb_to_xyz,
    lrgb_to_xyz_3f32,
    lrgb_to_xyz_3f64,
    lrgb_to_xyz_4f32,
    lrgb_to_xyz_4f64
);
cdef3!(
    xyz_to_cielab,
    xyz_to_cielab_3f32,
    xyz_to_cielab_3f64,
    xyz_to_cielab_4f32,
    xyz_to_cielab_4f64
);
cdef3!(
    xyz_to_oklab,
    xyz_to_oklab_3f32,
    xyz_to_oklab_3f64,
    xyz_to_oklab_4f32,
    xyz_to_oklab_4f64
);
cdef3!(
    xyz_to_jzazbz,
    xyz_to_jzazbz_3f32,
    xyz_to_jzazbz_3f64,
    xyz_to_jzazbz_4f32,
    xyz_to_jzazbz_4f64
);
cdef3!(
    lab_to_lch,
    lab_to_lch_3f32,
    lab_to_lch_3f64,
    lab_to_lch_4f32,
    lab_to_lch_4f64
);
cdef3!(
    _lrgb_to_ictcp,
    _lrgb_to_ictcp_3f32,
    _lrgb_to_ictcp_3f64,
    _lrgb_to_ictcp_4f32,
    _lrgb_to_ictcp_4f64
);

// Backward
cdef3!(
    hsv_to_srgb,
    hsv_to_srgb_3f32,
    hsv_to_srgb_3f64,
    hsv_to_srgb_4f32,
    hsv_to_srgb_4f64
);
cdef3!(
    lrgb_to_srgb,
    lrgb_to_srgb_3f32,
    lrgb_to_srgb_3f64,
    lrgb_to_srgb_4f32,
    lrgb_to_srgb_4f64
);
cdef3!(
    xyz_to_lrgb,
    xyz_to_lrgb_3f32,
    xyz_to_lrgb_3f64,
    xyz_to_lrgb_4f32,
    xyz_to_lrgb_4f64
);
cdef3!(
    cielab_to_xyz,
    cielab_to_xyz_3f32,
    cielab_to_xyz_3f64,
    cielab_to_xyz_4f32,
    cielab_to_xyz_4f64
);
cdef3!(
    oklab_to_xyz,
    oklab_to_xyz_3f32,
    oklab_to_xyz_3f64,
    oklab_to_xyz_4f32,
    oklab_to_xyz_4f64
);
cdef3!(
    jzazbz_to_xyz,
    jzazbz_to_xyz_3f32,
    jzazbz_to_xyz_3f64,
    jzazbz_to_xyz_4f32,
    jzazbz_to_xyz_4f64
);
cdef3!(
    lch_to_lab,
    lch_to_lab_3f32,
    lch_to_lab_3f64,
    lch_to_lab_4f32,
    lch_to_lab_4f64
);
cdef3!(
    _ictcp_to_lrgb,
    _ictcp_to_lrgb_3f32,
    _ictcp_to_lrgb_3f64,
    _ictcp_to_lrgb_4f32,
    _ictcp_to_lrgb_4f64
);

// }}}
