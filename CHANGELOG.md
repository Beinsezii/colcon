# 0.10.1
`str2space` FFI
## C LIB
 - `str2space` has an FFI wrapper now, similar to `convert_space`
  - `str2space_ffi` generic public function with C types as well
 - Renamed `convert_space_ffi_NT` -> `convert_space_NT`

# 0.10.0
Alpha channel "support"
## BREAKING
 - Most functions can now take 4 channels instead of 3 channels. While the compiler can usually infer which to use, you may have to manually annotate in some scenarios. This lets you directly process images with an alpha channel without splitting the data into separate streams
 - The external C functions have all been renamed to monotyped versions of the generics. The naming scheme is `generic_name_NDtype` where `N` is the channel count (3 or 4) and DType is currently either `f32` or `f64`. So the prior `xyz_to_oklab` now becomes `xyz_to_oklab_3f32`

## IMPROVEMENTS
 - Many functions are 10-50% faster due to optimized algorithms, particularly fast cube root.
   - Fast cube root is only different by an extremely tiny fraction
 - Some integer functions are now stable in `const`
 - `srgb_to_irgb` now properly rounds before casting
