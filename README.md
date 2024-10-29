# ColCon 0.10.1
Comprehensive colorspace conversions in Rust.

## Features
  * Pure Rust, no dependencies.
  * sRGB, RGB, CIE XYZ, CIE LAB, Oklab, JzAzBz, HSV
    + LCH/Cylindrical versions of all LAB spaces
  * Most functions compile to a C lib
  * Generic over F32/F64 with const alpha channel
  * FMA3 used where supported
  * Accurate across a wide variety of tests, referencing [colour-science](https://github.com/colour-science/colour)

## MSRV
|Colcon|Rust|
|-|-|
|0.10|1.65|
|`main`|1.82|

## Future
  * `std::simd` either after stabilization or as a nightly feature
  * More spaces?
    * iCtCp has extant conversion functions but no `Space::` option yet as colour-science is inconsistent as reference

## F.A.Q.
Question|Answer
---|---
Why?|I greatly enjoy working with Uniform Color Spaces and wish to see them become more accessible and easy to use.
