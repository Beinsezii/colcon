#! /usr/bin/env python3
import ctypes
from sys import platform

c_float3 = ctypes.c_float * 3
c_float_p = ctypes.POINTER(ctypes.c_float)

if platform == "win32":
    LIBRARY = "colcon.dll"
elif platform == "darwin":
    LIBRARY = "libcolcon.dylib"
elif platform == "linux":
    LIBRARY = "libcolcon.so"

colcon = ctypes.CDLL(f"./target/release/{LIBRARY}")

colcon.convert_space_3f32.argtypes = [
    ctypes.c_char_p,
    ctypes.c_char_p,
    c_float_p,
    ctypes.c_uint,
]
colcon.convert_space_3f32.restype = ctypes.c_int32

colcon.str2space_3f32.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
colcon.str2space_3f32.restype = c_float_p  # No way to have a known size?

# up
colcon.srgb_to_hsv_3f32.argtypes = [c_float3]
colcon.srgb_to_lrgb_3f32.argtypes = [c_float3]
colcon.lrgb_to_xyz_3f32.argtypes = [c_float3]
colcon.xyz_to_cielab_3f32.argtypes = [c_float3]
colcon.xyz_to_oklab_3f32.argtypes = [c_float3]
colcon.xyz_to_jzazbz_3f32.argtypes = [c_float3]
colcon.lab_to_lch_3f32.argtypes = [c_float3]

# down
colcon.lch_to_lab_3f32.argtypes = [c_float3]
colcon.jzazbz_to_xyz_3f32.argtypes = [c_float3]
colcon.oklab_to_xyz_3f32.argtypes = [c_float3]
colcon.cielab_to_xyz_3f32.argtypes = [c_float3]
colcon.xyz_to_lrgb_3f32.argtypes = [c_float3]
colcon.lrgb_to_srgb_3f32.argtypes = [c_float3]
colcon.srgb_to_hsv_3f32.argtypes = [c_float3]

# extra
colcon.srgb_eotf_f32.argtypes = [ctypes.c_float]
colcon.srgb_eotf_f32.restype = ctypes.c_float
colcon.srgb_oetf_f32.argtypes = [ctypes.c_float]
colcon.srgb_oetf_f32.restype = ctypes.c_float
colcon.pq_eotf_f32.argtypes = [ctypes.c_float]
colcon.pq_eotf_f32.restype = ctypes.c_float
colcon.pqz_eotf_f32.argtypes = [ctypes.c_float]
colcon.pqz_eotf_f32.restype = ctypes.c_float
colcon.pq_oetf_f32.argtypes = [ctypes.c_float]
colcon.pq_oetf_f32.restype = ctypes.c_float
colcon.pqz_oetf_f32.argtypes = [ctypes.c_float]
colcon.pqz_oetf_f32.restype = ctypes.c_float
colcon.hk_high2023_3f32.argtypes = [c_float3]
colcon.hk_high2023_comp_3f32.argtypes = [c_float3]

# other dtypes
colcon.srgb_to_lrgb_4f32.argtypes = [ctypes.c_float * 4]
colcon.srgb_to_lrgb_3f64.argtypes = [ctypes.c_double * 3]
colcon.srgb_to_lrgb_4f64.argtypes = [ctypes.c_double * 4]


SRGB = [0.20000000, 0.35000000, 0.95000000]
LRGB = [0.03310477, 0.10048151, 0.89000541]
HSV = [0.63333333, 0.78947368, 0.95000000]
XYZ = [0.21023057, 0.14316084, 0.85856646]
LAB = [44.68286380, 40.81934559, -80.13283179]
LCH = [44.68286380, 89.93047151, 296.99411238]
OKLAB = [0.53893206, -0.01239956, -0.23206808]
JZAZBZ = [0.00601244, -0.00145433, -0.01984568]


def pixcmp(a, b):
    epsilon = 1e-4
    for ac, bc in zip(a, b):
        if abs(ac - bc) > epsilon:
            print(
                f"\nFAIL:\n[{a[0]:.8f}, {a[1]:.8f}, {a[2]:.8f}]\n[{b[0]:.8f}, {b[1]:.8f}, {b[2]:.8f}]\n"
            )
            break


# up
pix = c_float3(*SRGB)
colcon.srgb_to_hsv_3f32(pix)
pixcmp(list(pix), HSV)

pix = c_float3(*SRGB)
colcon.srgb_to_lrgb_3f32(pix)
pixcmp(list(pix), LRGB)

pix = c_float3(*LRGB)
colcon.lrgb_to_xyz_3f32(pix)
pixcmp(list(pix), XYZ)

pix = c_float3(*XYZ)
colcon.xyz_to_cielab_3f32(pix)
pixcmp(list(pix), LAB)

pix = c_float3(*XYZ)
colcon.xyz_to_oklab_3f32(pix)
pixcmp(list(pix), OKLAB)

pix = c_float3(*XYZ)
colcon.xyz_to_jzazbz_3f32(pix)
pixcmp(list(pix), JZAZBZ)

pix = c_float3(*LAB)
colcon.lab_to_lch_3f32(pix)
pixcmp(list(pix), LCH)

# down
pix = c_float3(*LCH)
colcon.lch_to_lab_3f32(pix)
pixcmp(list(pix), LAB)

pix = c_float3(*LAB)
colcon.cielab_to_xyz_3f32(pix)
pixcmp(list(pix), XYZ)

pix = c_float3(*JZAZBZ)
colcon.jzazbz_to_xyz_3f32(pix)
pixcmp(list(pix), XYZ)

pix = c_float3(*OKLAB)
colcon.oklab_to_xyz_3f32(pix)
pixcmp(list(pix), XYZ)

pix = c_float3(*XYZ)
colcon.xyz_to_lrgb_3f32(pix)
pixcmp(list(pix), LRGB)

pix = c_float3(*LRGB)
colcon.lrgb_to_srgb_3f32(pix)
pixcmp(list(pix), SRGB)

pix = c_float3(*SRGB)
colcon.srgb_to_hsv_3f32(pix)
pixcmp(list(pix), HSV)

pix = (ctypes.c_float * len(SRGB))(*SRGB)
if colcon.convert_space_3f32("srgb".encode(), "lch".encode(), pix, len(pix)) != 0:
    print("CONVERT SPACE FAIL")
pixcmp(list(pix), LCH)

pix = colcon.str2space_3f32(f"oklab {OKLAB}".encode(), "srgb".encode())
pixcmp(pix[0:3], SRGB)
# validate null is utilized
assert not bool(colcon.str2space_3f32("cheese sandwhich".encode(), "srgb".encode()))
