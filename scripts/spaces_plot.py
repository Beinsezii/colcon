#! /usr/bin/env python3
import colour
import numpy as np
import matplotlib as mpl

values = [[[65, c, h] for h in range(0, 361)] for c in range(0, 101)]

hsv = colour.HSV_to_RGB(np.array([[[h / 360, c / 100, 0.65] for h in range(0, 361)] for c in range(0, 101)]))
lab = colour.XYZ_to_sRGB(colour.Lab_to_XYZ(colour.LCHab_to_Lab(np.array(values))))
oklab = colour.XYZ_to_sRGB(colour.Oklab_to_XYZ(colour.LCHab_to_Lab(np.array(values) / np.array([100, 400, 1]))))
cam16ucs = colour.XYZ_to_sRGB(colour.CAM16UCS_to_XYZ(colour.LCHab_to_Lab(np.array(values) / np.array([1, 2.5, 1]))))
ictcp = colour.XYZ_to_sRGB(colour.ICtCp_to_XYZ(colour.LCHab_to_Lab(np.array(values) / np.array([650, 1250, 1]))))
jzazbz = colour.XYZ_to_sRGB(colour.JzAzBz_to_XYZ(colour.LCHab_to_Lab(np.array(values) / np.array([6500, 6500, 1]))))

plots = [
    (hsv, 'HSV'),
    (lab, 'CIE LAB'),
    (oklab, 'Oklab'),
    (cam16ucs, 'CAM16-UCS'),
    (ictcp, 'ICtCp'),
    (jzazbz, 'JzAzBz'),
]

figs = mpl.pyplot.figure().subplots(nrows=len(plots), sharex=True, sharey=True, gridspec_kw={'hspace': 0.5})

for ((im, title), fig) in zip(plots, figs):
    fig.axes.imshow(im)
    fig.axes.set_title(title)
    fig.axes.set_ylim(bottom=0, top=100)

mpl.pyplot.show()
