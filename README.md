# ColCon 0.9.0
Simple colorspace conversions in Rust.

## Features
  * Pure Rust, no dependencies.
  * sRGB, RGB, CIE XYZ, CIE LAB, Oklab, JzAzBz, HSV
    + LCH/Cylindrical versions of all LAB spaces
  * Most functions compile to a C lib
  * Generic over F32/F64
  * FMA3 used where supported
  * Accurate across a wide variety of tests, referencing [colour-science](https://github.com/colour-science/colour)

## Future
  * Look into SIMD when supported by standard library
  * More spaces?

## Known Issues
  * `convert_space_sliced` is slower than it could be. Waiting for [slice_as_chunks](https://github.com/rust-lang/rust/issues/74985) to land in stable.
  * Performing many (>100) conversions in sequence will gradually degrade the data due to tiny precision issues accumulating.

## F.A.Q.
Question|Answer
---|---
Why?|I just wanna say "go from this to this" without any fuss.
