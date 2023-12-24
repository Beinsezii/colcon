# ColCon 0.5.0
Simple colorspace conversions in Rust.

## Features
  * Pure Rust, no dependencies.
  * Most functions compile to a C lib
  * sRGB, HSV, CIE XYZ, CIE L*a*b*, Oklab, JzAzBz
    + Cylindrical versions of all LAB-adjacent spaces
  * Accurate to 1e-5 minimum, referencing [colour-science](https://github.com/colour-science/colour)

## Future
  * Look into SIMD when supported by standard library
  * More spaces?
  * Generic dtypes?

## Known Issues
  * `convert_space_sliced` is slower than it could be. Waiting for [slice_as_chunks](https://github.com/rust-lang/rust/issues/74985) to land in stable.
  * Performing many (>100) conversions in sequence will gradually degrade the data due to tiny precision issues accumulating.

## F.A.Q.
Question|Answer
---|---
Why?|I just wanna say "go from this to this" without any fuss.
