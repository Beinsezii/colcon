#! /usr/bin/env python3
import colour
import numpy as np
import time

import ctypes
from sys import platform

if platform == "win32":
    LIBRARY = "colcon.dll"
elif platform == "darwin":
    LIBRARY = "libcolcon.dylib"
elif platform == "linux":
    LIBRARY = "libcolcon.so"

colcon = ctypes.CDLL(f"./target/release/{LIBRARY}")

colcon.convert_space_ffi.argtypes = [
    ctypes.c_char_p,
    ctypes.c_char_p,
    np.ctypeslib.ndpointer(ndim=1, flags=("W", "C", "A")),
    ctypes.c_uint,
]
colcon.convert_space_ffi.restype = ctypes.c_int32

img = np.random.rand(2048, 2048, 3).astype(np.float32)
img2 = img.copy().flatten()

now = time.perf_counter_ns()
colcon.convert_space_ffi("SRGB".encode("UTF-8"), "CIELCH".encode("UTF-8"), img2, img2.nbytes // 4)
print("Colcon: ", (time.perf_counter_ns() - now) // 1000 // 1000, "ms")

now = time.perf_counter_ns()
colour.sRGB_to_XYZ(img)
colour.XYZ_to_Lab(img)
colour.Lab_to_LCHab(img)
print("NumPy: ", (time.perf_counter_ns() - now) // 1000 // 1000, "ms")
