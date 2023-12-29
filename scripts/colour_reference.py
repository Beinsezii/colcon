#! /usr/bin/env python3
import colour
import numpy as np

def rustprint(id, arr):
    layers = ""
    for a in arr:
        layers += f"    [{a[0]:.8f}, {a[1]:.8f}, {a[2]:.8f}],\n"
    print(f"const {id.upper()}: &'static [[f32; 3]] = &[\n{layers}];")

d65 = colour.xyY_to_XYZ(colour.xy_to_xyY(colour.CCS_ILLUMINANTS["CIE 1931 2 Degree Standard Observer"]["D65"]))
srgb = np.array([
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [1.0, 1.0, 0.0],
    [0.0, 1.0, 1.0],
    [1.0, 0.0, 1.0],
    [1.0, 1.0, 1.0],
    [5.0, 10.0, 15.0],
    [-5.0, -10.0, -15.0],
    ])
lrgb = colour.models.eotf_sRGB(srgb)
hsv = colour.RGB_to_HSV(srgb)
xyz = colour.sRGB_to_XYZ(srgb)
lab = colour.XYZ_to_Lab(xyz)
lch = colour.Lab_to_LCHab(lab)
oklab = colour.XYZ_to_Oklab(xyz)
oklch = colour.Lab_to_LCHab(oklab)
jzazbz = colour.XYZ_to_JzAzBz(xyz)
jzczhz = colour.Lab_to_LCHab(jzazbz)
cam16ucs = colour.XYZ_to_CAM16UCS(xyz)
# Something's messed up here. Totally different results depending on how you reach the ICtCp space...
bt2020 = colour.RGB_to_RGB(srgb, input_colourspace='sRGB', output_colourspace='ITU-R BT.2020')
ictcp = colour.RGB_to_ICtCp(bt2020, method='ITU-R BT.2100-2 PQ')
ictcp2 = colour.XYZ_to_ICtCp(xyz, method='ITU-R BT.2100-2 PQ')

print(f"pub const D65: [f32; 3] = [{d65[0]}, {d65[1]}, {d65[2]}];")

print()

rustprint('srgb', srgb)
rustprint('lrgb', lrgb)
rustprint('hsv', hsv)
rustprint('xyz', xyz)
rustprint('lab', lab)
rustprint('lch', lch)
rustprint('oklab', oklab)
rustprint('oklch', oklch)
rustprint('jzazbz', jzazbz)
rustprint('jzczhz', jzczhz)
rustprint('cam16ucs', cam16ucs)
rustprint('bt2020', bt2020)
rustprint('ictcp', ictcp)
rustprint('ictcp2', ictcp2)

pyprint = lambda id, arr: print(f"{id.upper()} = [{arr[0]:.8f}, {arr[1]:.8f}, {arr[2]:.8f}]")
print()

pyprint('srgb', srgb)
pyprint('lrgb', lrgb)
pyprint('hsv', hsv)
pyprint('xyz', xyz)
pyprint('lab', lab)
pyprint('lch', lch)
pyprint('oklab', oklab)
pyprint('jzazbz', jzazbz)
