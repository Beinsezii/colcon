#! /usr/bin/env python3
import ctypes
from sys import platform
cpixel = ctypes.c_float * 3
cpixels = ctypes.POINTER(ctypes.c_float)

if platform == "win32":
    LIBRARY = "colcon.dll"
elif platform == "darwin":
    LIBRARY = "libcolcon.dylib"
elif platform == "linux":
    LIBRARY = "libcolcon.so"

colcon = ctypes.CDLL(f"./target/release/{LIBRARY}")

colcon.convert_space_ffi.argtypes = [ctypes.c_char_p, ctypes.c_char_p, cpixels, ctypes.c_uint]
colcon.convert_space_ffi.restype = ctypes.c_int32

# up
colcon.srgb_to_hsv.argtypes = [cpixel]
colcon.srgb_to_lrgb.argtypes = [cpixel]
colcon.lrgb_to_xyz.argtypes = [cpixel]
colcon.xyz_to_lab.argtypes = [cpixel]
colcon.xyz_to_oklab.argtypes = [cpixel]
colcon.lab_to_lch.argtypes = [cpixel]

# down
colcon.lch_to_lab.argtypes = [cpixel]
colcon.oklab_to_xyz.argtypes = [cpixel]
colcon.lab_to_xyz.argtypes = [cpixel]
colcon.xyz_to_lrgb.argtypes = [cpixel]
colcon.lrgb_to_srgb.argtypes = [cpixel]
colcon.srgb_to_hsv.argtypes = [cpixel]

# extra
colcon.expand_gamma.argtypes = [ctypes.c_float]
colcon.expand_gamma.restype = ctypes.c_float
colcon.correct_gamma.argtypes = [ctypes.c_float]
colcon.correct_gamma.restype = ctypes.c_float
colcon.hk_high2023.argtypes = [cpixel]
colcon.hk_high2023_comp.argtypes = [cpixel]

SRGB = [0.20000000, 0.35000000, 0.95000000]
LRGB = [0.03310477, 0.10048151, 0.89000541]
HSV = [0.63333333, 0.78947368, 0.95000000]
XYZ = [0.21023057, 0.14316084, 0.85856646]
LAB = [44.68286380, 40.81934559, -80.13283179]
LCH = [44.68286380, 89.93047151, 296.99411238]
OKLAB = [0.53893206, -0.01239956, -0.23206808]

def pixcmp(a, b):
    epsilon = 1e-5
    for (ac, bc) in zip(a, b):
        if abs(ac - bc) > epsilon:
            print(f"\nFAIL:\n[{a[0]:.8f}, {a[1]:.8f}, {a[2]:.8f}]\n[{b[0]:.8f}, {b[1]:.8f}, {b[2]:.8f}]\n")
            break

# up
pix = cpixel(*SRGB)
colcon.srgb_to_hsv(pix)
pixcmp(list(pix), HSV)

pix = cpixel(*SRGB)
colcon.srgb_to_lrgb(pix)
pixcmp(list(pix), LRGB)

pix = cpixel(*LRGB)
colcon.lrgb_to_xyz(pix)
pixcmp(list(pix), XYZ)

pix = cpixel(*XYZ)
colcon.xyz_to_lab(pix)
pixcmp(list(pix), LAB)

pix = cpixel(*XYZ)
colcon.xyz_to_oklab(pix)
pixcmp(list(pix), OKLAB)

pix = cpixel(*LAB)
colcon.lab_to_lch(pix)
pixcmp(list(pix), LCH)

# down
pix = cpixel(*LCH)
colcon.lch_to_lab(pix)
pixcmp(list(pix), LAB)

pix = cpixel(*LAB)
colcon.lab_to_xyz(pix)
pixcmp(list(pix), XYZ)

pix = cpixel(*OKLAB)
colcon.oklab_to_xyz(pix)
pixcmp(list(pix), XYZ)

pix = cpixel(*XYZ)
colcon.xyz_to_lrgb(pix)
pixcmp(list(pix), LRGB)

pix = cpixel(*LRGB)
colcon.lrgb_to_srgb(pix)
pixcmp(list(pix), SRGB)

pix = cpixel(*SRGB)
colcon.srgb_to_hsv(pix)
pixcmp(list(pix), HSV)

pix = (ctypes.c_float * len(SRGB))(*SRGB)
if colcon.convert_space_ffi("srgb".encode(), "lch".encode(), pix, len(pix)) != 0:
    print("CONVERT SPACE FAIL")
pixcmp(list(pix), LCH)
