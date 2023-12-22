#! /usr/bin/env python3
import colour
import numpy as np

d65 = colour.xyY_to_XYZ(colour.xy_to_xyY(colour.CCS_ILLUMINANTS["CIE 1931 2 Degree Standard Observer"]["D65"]))
srgb = np.array([0.2, 0.35, 0.95], dtype=np.float64)
lrgb = colour.models.eotf_sRGB(srgb)
hsv = colour.RGB_to_HSV(srgb)
xyz = colour.sRGB_to_XYZ(srgb)
lab = colour.XYZ_to_Lab(xyz)
lch = colour.Lab_to_LCHab(lab)
oklab = colour.XYZ_to_Oklab(xyz)
cam16ucs = colour.XYZ_to_CAM16UCS(xyz)
ictcp = colour.XYZ_to_ICtCp(xyz)
jzazbz = colour.XYZ_to_JzAzBz(xyz)

print(f"pub const D65: [f32; 3] = [{d65[0]}, {d65[1]}, {d65[2]}];")

rustprint = lambda id, arr: print(f"const {id.upper()}: [f32; 3] = [{arr[0]:.8f}, {arr[1]:.8f}, {arr[2]:.8f}];")
print()

rustprint('srgb', srgb)
rustprint('lrgb', lrgb)
rustprint('hsv', hsv)
rustprint('xyz', xyz)
rustprint('lab', lab)
rustprint('lch', lch)
rustprint('oklab', oklab)
rustprint('cam16ucs', cam16ucs)
rustprint('ictcp', ictcp)
rustprint('jzazbz', jzazbz)

pyprint = lambda id, arr: print(f"{id.upper()} = [{arr[0]:.8f}, {arr[1]:.8f}, {arr[2]:.8f}]")
print()

pyprint('srgb', srgb)
pyprint('lrgb', lrgb)
pyprint('hsv', hsv)
pyprint('xyz', xyz)
pyprint('lab', lab)
pyprint('lch', lch)
pyprint('oklab', oklab)
pyprint('cam16ucs', cam16ucs)
pyprint('ictcp', ictcp)
pyprint('jzazbz', jzazbz)
