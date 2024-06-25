use super::*;

const HEX: &str = "#3359F2";
const IRGB: [u8; 3] = [51, 89, 242];

const HEXA: &str = "#3359F259";
const IRGBA: [u8; 4] = [51, 89, 242, 89];

// ### COLOUR-REFS ### {{{

const SRGB: &'static [[f64; 3]] = &[
    [0.00000000, 0.00000000, 0.00000000],
    [1.00000000, 0.00000000, 0.00000000],
    [0.00000000, 1.00000000, 0.00000000],
    [0.00000000, 0.00000000, 1.00000000],
    [1.00000000, 1.00000000, 0.00000000],
    [0.00000000, 1.00000000, 1.00000000],
    [1.00000000, 0.00000000, 1.00000000],
    [1.00000000, 1.00000000, 1.00000000],
    [5.00000000, 10.00000000, 15.00000000],
    [-5.00000000, -10.00000000, -15.00000000],
];
const LRGB: &'static [[f64; 3]] = &[
    [0.00000000, 0.00000000, 0.00000000],
    [1.00000000, 0.00000000, 0.00000000],
    [0.00000000, 1.00000000, 0.00000000],
    [0.00000000, 0.00000000, 1.00000000],
    [1.00000000, 1.00000000, 0.00000000],
    [0.00000000, 1.00000000, 1.00000000],
    [1.00000000, 0.00000000, 1.00000000],
    [1.00000000, 1.00000000, 1.00000000],
    [42.96599571, 223.82627997, 589.69564509],
    [-0.38699690, -0.77399381, -1.16099071],
];
const HSV: &'static [[f64; 3]] = &[
    [0.00000000, 0.00000000, 0.00000000],
    [0.00000000, 1.00000000, 1.00000000],
    [0.33333333, 1.00000000, 1.00000000],
    [0.66666667, 1.00000000, 1.00000000],
    [0.16666667, 1.00000000, 1.00000000],
    [0.50000000, 1.00000000, 1.00000000],
    [0.83333333, 1.00000000, 1.00000000],
    [0.00000000, 0.00000000, 1.00000000],
    [0.58333333, 0.66666667, 15.00000000],
    [0.08333333, -2.00000000, -5.00000000],
];
const XYZ: &'static [[f64; 3]] = &[
    [0.00000000, 0.00000000, 0.00000000],
    [0.41240000, 0.21260000, 0.01930000],
    [0.35760000, 0.71520000, 0.11920000],
    [0.18050000, 0.07220000, 0.95050000],
    [0.77000000, 0.92780000, 0.13850000],
    [0.53810000, 0.78740000, 1.06970000],
    [0.59290000, 0.28480000, 0.96980000],
    [0.95050000, 1.00000000, 1.08900000],
    [204.19951828, 211.79115169, 588.01504694],
    [-0.64593653, -0.71965944, -1.20325077],
];
const CIELAB: &'static [[f64; 3]] = &[
    [0.00000000, 0.00000000, 0.00000000],
    [53.23288179, 80.11117774, 67.22370367],
    [87.73703347, -86.18285500, 83.18783466],
    [32.30258667, 79.19808023, -107.85035570],
    [97.13824698, -21.55360786, 94.48949749],
    [91.11652111, -48.07757700, -14.12426716],
    [60.31993366, 98.25632722, -60.82956929],
    [100.00000000, 0.00772827, 0.00353528],
    [675.44970111, 14.25078120, -436.42562428],
    [-650.06570921, 155.94479927, 599.90623227],
];
const LCH: &'static [[f64; 3]] = &[
    [0.00000000, 0.00000000, 0.00000000],
    [53.23288179, 104.57928635, 40.00102571],
    [87.73703347, 119.78188649, 136.01306869],
    [32.30258667, 133.80596077, 306.29106810],
    [97.13824698, 96.91657829, 102.84964820],
    [91.11652111, 50.10936373, 196.37177336],
    [60.31993366, 115.56185503, 328.23873929],
    [100.00000000, 0.00849849, 24.58159697],
    [675.44970111, 436.65823054, 271.87023758],
    [-650.06570921, 619.84374477, 75.42854110],
];
const OKLAB: &'static [[f64; 3]] = &[
    [0.00000000, 0.00000000, 0.00000000],
    [0.62792590, 0.22488760, 0.12580493],
    [0.86645187, -0.23392144, 0.17942177],
    [0.45203295, -0.03235164, -0.31162054],
    [0.96798108, -0.07139347, 0.19848985],
    [0.90541467, -0.14944654, -0.03950465],
    [0.70165739, 0.27462625, -0.16926875],
    [1.00000174, 0.00000229, -0.00011365],
    [5.95611678, -0.42728383, -1.24134000],
    [-0.89252901, 0.04256306, 0.07613246],
];
const OKLCH: &'static [[f64; 3]] = &[
    [0.00000000, 0.00000000, 0.00000000],
    [0.62792590, 0.25768453, 29.22319405],
    [0.86645187, 0.29480741, 142.51117284],
    [0.45203295, 0.31329538, 264.07293384],
    [0.96798108, 0.21093897, 109.78280773],
    [0.90541467, 0.15457971, 194.80686888],
    [0.70165739, 0.32260113, 328.35196366],
    [1.00000174, 0.00011368, 271.15202477],
    [5.95611678, 1.31282005, 251.00593438],
    [-0.89252901, 0.08722251, 60.79193305],
];
const JZAZBZ: &'static [[f64; 3]] = &[
    [0.00000000, 0.00000000, 0.00000000],
    [0.00816620, 0.01616207, 0.01140765],
    [0.01243416, -0.01624847, 0.01656722],
    [0.00478354, -0.00116064, -0.02495001],
    [0.01611356, -0.00372360, 0.02080003],
    [0.01418671, -0.01169390, -0.00544011],
    [0.01049327, 0.01640515, -0.01404140],
    [0.01758021, -0.00002806, -0.00002067],
    [0.22861137, -0.04674604, -0.11322403],
    [-0.78600741, 1933.15497262, 2113.40419865],
];
const JZCZHZ: &'static [[f64; 3]] = &[
    [0.00000000, 0.00000000, 0.00000000],
    [0.00816620, 0.01978250, 35.21553828],
    [0.01243416, 0.02320529, 134.44348766],
    [0.00478354, 0.02497699, 267.33659019],
    [0.01611356, 0.02113070, 100.14952867],
    [0.01418671, 0.01289736, 204.94830520],
    [0.01049327, 0.02159375, 319.43931639],
    [0.01758021, 0.00003485, 216.37796402],
    [0.22861137, 0.12249438, 247.56607117],
    [-0.78600741, 2864.18670045, 47.55048725],
];

const _ICTCP2: &'static [[f64; 3]] = &[
    [0.00000073, -0.00000000, 0.00000000],
    [0.08575747, -0.02634122, 0.09894511],
    [0.13074534, -0.11285245, -0.02347905],
    [0.06172962, 0.10943968, -0.05098289],
    [0.14470233, -0.10333260, 0.01679849],
    [0.13713260, -0.00256371, -0.03877447],
    [0.09966927, 0.10169935, 0.04737088],
    [0.14994586, -0.00000057, 0.00000710],
    [0.58829375, 0.11054605, -0.09104248],
    [140.00646866, -38.81567400, 54.38089910],
];

// ### COLOUR-REFS ### }}}

// ### Comparison FNs ### {{{
fn pix_cmp(input: &[[f64; 3]], reference: &[[f64; 3]], epsilon: f64, skips: &'static [usize]) {
    let mut err = String::new();
    let mut cum_err = 0.0;
    for (n, (i, r)) in input.iter().zip(reference.iter()).enumerate() {
        if skips.contains(&n) {
            continue;
        }
        for (a, b) in i.iter().zip(r.iter()) {
            if (a - b).abs() > epsilon || !a.is_finite() || !b.is_finite() {
                let dev = i
                    .iter()
                    .zip(r.iter())
                    .map(|(ix, rx)| ((ix - rx) + 1.0).abs().powi(2))
                    .sum::<f64>();
                err.push_str(&format!(
                    "\nA{n}: {:.8} {:.8} {:.8}\nB{n}: {:.8} {:.8} {:.8}\nERR²: {}\n",
                    i[0], i[1], i[2], r[0], r[1], r[2], dev
                ));
                if dev.is_finite() {
                    cum_err += dev
                };
                break;
            }
        }
    }
    if !err.is_empty() {
        panic!("{}\nCUM ERR²: {}", err, cum_err)
    }
}

fn func_cmp_full(
    input: &[[f64; 3]],
    reference: &[[f64; 3]],
    function: fn(&mut [f64; 3]),
    epsilon: f64,
    skips: &'static [usize],
) {
    let mut input = input.to_owned();
    input.iter_mut().for_each(|p| function(p));
    pix_cmp(&input, reference, epsilon, skips);
}

fn func_cmp(input: &[[f64; 3]], reference: &[[f64; 3]], function: fn(&mut [f64; 3])) {
    func_cmp_full(input, reference, function, 1e-3, &[])
}

fn conv_cmp_full(
    input_space: Space,
    input: &[[f64; 3]],
    reference_space: Space,
    reference: &[[f64; 3]],
    epsilon: f64,
    skips: &'static [usize],
) {
    let mut input = input.to_owned();
    convert_space_chunked(input_space, reference_space, &mut input);
    pix_cmp(&input, reference, epsilon, skips)
}

fn conv_cmp(input_space: Space, input: &[[f64; 3]], reference_space: Space, reference: &[[f64; 3]]) {
    // skip places where hue can wrap
    conv_cmp_full(input_space, input, reference_space, reference, 1e-3, &[0, 1, 7])
}
// ### Comparison FNs ### }}}

// ### Single FN Accuracy ### {{{
#[test]
fn irgb_to() {
    assert_eq!(IRGB, srgb_to_irgb([0.2, 0.35, 0.95]))
}

#[test]
fn irgb_to_alpha() {
    assert_eq!(IRGBA, srgb_to_irgb([0.2, 0.35, 0.95, 0.35]))
}

#[test]
fn irgb_from() {
    let mut srgb = irgb_to_srgb::<f32, 3>(IRGB);
    // Round decimal to hundredths
    srgb.iter_mut().for_each(|c| *c = (*c * 100.0).round() / 100.0);
    assert_eq!([0.2, 0.35, 0.95], srgb)
}

#[test]
fn irgb_from_alpha() {
    let mut srgb = irgb_to_srgb::<f32, 4>(IRGBA);
    // Round decimal to hundredths
    srgb.iter_mut().for_each(|c| *c = (*c * 100.0).round() / 100.0);
    assert_eq!([0.2, 0.35, 0.95, 0.35], srgb)
}

#[test]
fn hex_to() {
    assert_eq!(HEX, irgb_to_hex(IRGB))
}

#[test]
fn hex_to_alpha() {
    assert_eq!(HEXA, irgb_to_hex(IRGBA))
}

#[test]
fn hex_from() {
    assert_eq!(IRGB, hex_to_irgb(HEX).unwrap());
    assert_eq!(IRGB, hex_to_irgb(HEXA).unwrap());
}

#[test]
fn hex_from_alpha() {
    assert_eq!(
        [IRGB[0], IRGB[1], IRGB[2], 123],
        hex_to_irgb_default::<4, 123>(HEX).unwrap()
    );
    assert_eq!(IRGBA, hex_to_irgb(HEXA).unwrap());
}

#[test]
fn hex_validations() {
    for hex in [
        "#ABCDEF",
        "#abcdef",
        "#ABCDEF01",
        "#abcdef01",
        "#ABCDEF",
        "ABCDEF",
        "  ABCDEF     ",
        "  #ABCDEF     ",
    ] {
        assert!(hex_to_irgb::<3>(hex).is_ok(), "NOT VALID 3: '{}'", hex);
        assert!(hex_to_irgb::<4>(hex).is_ok(), "NOT VALID 4: '{}'", hex);
    }
    for hex in [
        "", "#", "#5F", "#ABCDEG", "#abcdeg", "#ABCDEFF", "#abcdeg", "##ABCDEF", "ABCDEF#",
    ] {
        assert!(hex_to_irgb::<3>(hex).is_err(), "NOT INVALID 3: '{}'", hex);
        assert!(hex_to_irgb::<4>(hex).is_err(), "NOT INVALID 4: '{}'", hex);
    }
}

#[test]
fn hsv_forwards() {
    func_cmp(SRGB, HSV, srgb_to_hsv)
}
#[test]
fn hsv_backwards() {
    func_cmp(HSV, SRGB, hsv_to_srgb)
}

#[test]
fn lrgb_forwards() {
    func_cmp(SRGB, LRGB, srgb_to_lrgb)
}
#[test]
fn lrgb_backwards() {
    func_cmp(LRGB, SRGB, lrgb_to_srgb)
}

#[test]
fn xyz_forwards() {
    func_cmp(LRGB, XYZ, lrgb_to_xyz)
}
#[test]
fn xyz_backwards() {
    func_cmp(XYZ, LRGB, xyz_to_lrgb)
}

#[test]
fn lab_forwards() {
    func_cmp(XYZ, CIELAB, xyz_to_cielab)
}
#[test]
fn lab_backwards() {
    func_cmp(CIELAB, XYZ, cielab_to_xyz)
}

#[test]
fn lch_forwards() {
    func_cmp(CIELAB, LCH, lab_to_lch)
}
#[test]
fn lch_backwards() {
    func_cmp(LCH, CIELAB, lch_to_lab)
}

#[test]
fn oklab_forwards() {
    func_cmp(XYZ, OKLAB, xyz_to_oklab)
}
#[test]
fn oklab_backwards() {
    func_cmp(OKLAB, XYZ, oklab_to_xyz)
}

// Lower epsilon because of the extremely wide gamut creating tiny values
#[test]
fn jzazbz_forwards() {
    func_cmp_full(XYZ, JZAZBZ, xyz_to_jzazbz, 1e-2, &[])
}
#[test]
fn jzazbz_backwards() {
    func_cmp(JZAZBZ, XYZ, jzazbz_to_xyz)
}

#[test]
fn inversions() {
    let runs: &[(&[[f64; 3]], fn(pixel: &mut [f64; 3]), fn(pixel: &mut [f64; 3]), &str)] = &[
        (SRGB, srgb_to_hsv, hsv_to_srgb, "HSV"),
        (SRGB, srgb_to_lrgb, lrgb_to_srgb, "LRGB"),
        (LRGB, lrgb_to_xyz, xyz_to_lrgb, "XYZ"),         // 1e-4
        (LRGB, _lrgb_to_ictcp, _ictcp_to_lrgb, "ICTCP"), // 1e-4
        (XYZ, xyz_to_cielab, cielab_to_xyz, "CIELAB"),
        (XYZ, xyz_to_oklab, oklab_to_xyz, "OKLAB"),    // 1e-3
        (XYZ, xyz_to_jzazbz, jzazbz_to_xyz, "JZAZBZ"), // 1e-4
        (CIELAB, lab_to_lch, lch_to_lab, "LCH"),
    ];
    for (pixel, fwd, bwd, label) in runs.iter() {
        let mut owned = pixel.to_vec();
        owned.iter_mut().for_each(|p| {
            fwd(p);
            bwd(p);
        });
        println!("TEST {} INVERSION", label);
        pix_cmp(&owned, pixel, 1e-3, &[]);
    }
}
// Disable reference tests for public commits
//
// #[test]
// fn ictcp_forwards() {
//     func_cmp(LRGB, _ICTCP2, _lrgb_to_ictcp)
// }
// #[test]
// fn ictcp_backwards() {
//     func_cmp(_ICTCP2, LRGB, _ictcp_to_lrgb)
// }
// ### Single FN Accuracy ### }}}

/// ### Other Tests ### {{{
#[test]
fn tree_jump() {
    // forwards
    println!("HSV -> LCH");
    conv_cmp(Space::HSV, HSV, Space::CIELCH, LCH);

    println!("LCH -> OKLCH");
    conv_cmp(Space::CIELCH, LCH, Space::OKLCH, OKLCH);

    println!("OKLCH -> JZCZHZ");
    conv_cmp(Space::OKLCH, OKLCH, Space::JZCZHZ, JZCZHZ);

    println!("JZCZHZ -> HSV");
    conv_cmp(Space::JZCZHZ, JZCZHZ, Space::HSV, HSV);

    // backwards
    println!("HSV -> JZCZHZ");
    conv_cmp(Space::HSV, HSV, Space::JZCZHZ, JZCZHZ);

    println!("JZCZHZ -> OKLCH");
    conv_cmp(Space::JZCZHZ, JZCZHZ, Space::OKLCH, OKLCH);

    println!("OKLCH -> LCH");
    conv_cmp(Space::OKLCH, OKLCH, Space::CIELCH, LCH);

    // add 1 to skip because the hue wraps from 0.0000 to 0.9999
    // fuck you precision
    println!("LCH -> HSV");
    conv_cmp(Space::CIELCH, LCH, Space::HSV, HSV);
}

#[test]
fn alpha_untouch() {
    let mut pixel = [1.0, 2.0, 3.0, 4.0f64];
    for f in [
        srgb_to_hsv,
        hsv_to_srgb,
        srgb_to_lrgb,
        lrgb_to_xyz,
        xyz_to_cielab,
        xyz_to_oklab,
        xyz_to_jzazbz,
        lab_to_lch,
        _lrgb_to_ictcp,
        _ictcp_to_lrgb,
        lrgb_to_srgb,
        xyz_to_lrgb,
        cielab_to_xyz,
        oklab_to_xyz,
        jzazbz_to_xyz,
        lch_to_lab,
    ] {
        f(&mut pixel);
        assert_eq!(pixel[3].to_bits(), 4.0_f64.to_bits(), "{:?}", f);
    }
    convert_space(Space::SRGB, Space::CIELCH, &mut pixel);
    assert_eq!(pixel[3].to_bits(), 4.0_f64.to_bits());
    let mut chunks = [pixel, pixel, pixel];
    convert_space_chunked(Space::CIELCH, Space::SRGB, &mut chunks);
    chunks
        .iter()
        .for_each(|c| assert_eq!(c[3].to_bits(), 4.0_f64.to_bits(), "alpha_untouch_chunked"));
    let mut slice = [pixel, pixel, pixel].iter().fold(Vec::<f64>::new(), |mut acc, it| {
        acc.extend_from_slice(it.as_slice());
        acc
    });
    convert_space_sliced::<_, 4>(Space::CIELCH, Space::SRGB, &mut slice);
    slice
        .iter()
        .skip(3)
        .step_by(4)
        .for_each(|n| assert_eq!(n.to_bits(), 4.0_f64.to_bits(), "alpha_untouch_sliced"));
}

#[test]
fn sliced() {
    let mut pixel: Vec<f64> = SRGB.iter().fold(Vec::new(), |mut acc, it| {
        acc.extend_from_slice(it);
        acc
    });
    convert_space_sliced::<_, 3>(Space::SRGB, Space::CIELCH, &mut pixel);
    pix_cmp(
        &pixel
            .chunks_exact(3)
            .map(|c| c.try_into().unwrap())
            .collect::<Vec<[f64; 3]>>(),
        LCH,
        1e-2,
        &[],
    );
}

#[test]
fn sliced_odd() {
    let mut pixel: Vec<f64> = SRGB.iter().fold(Vec::new(), |mut acc, it| {
        acc.extend_from_slice(it);
        acc
    });
    pixel.push(1234.5678);
    convert_space_sliced::<_, 3>(Space::SRGB, Space::CIELCH, &mut pixel);
    pix_cmp(
        &pixel
            .chunks_exact(3)
            .map(|c| c.try_into().unwrap())
            .collect::<Vec<[f64; 3]>>(),
        LCH,
        1e-2,
        &[],
    );
    assert_eq!(*pixel.last().unwrap(), 1234.5678);
}

#[test]
fn sliced_smol() {
    let pixels = [1.0, 0.0];
    let mut smol = pixels.clone();
    convert_space_sliced::<_, 3>(Space::SRGB, Space::CIELCH, &mut smol);
    assert_eq!(pixels, smol);
}

#[test]
fn interweave() {
    let srgb: Vec<[f32; 3]> = SRGB.iter().map(|p| p.map(|c| c as f32)).collect();
    let slice: Vec<f32> = srgb.iter().fold(Vec::new(), |mut acc, it| {
        acc.extend_from_slice(it);
        acc
    });
    let mut new = slice.clone();
    new.push(1234.5678);

    let deinterleaved = unweave::<3>(&new);
    assert_eq!(deinterleaved[0].len(), deinterleaved[1].len());
    assert_eq!(deinterleaved[0].len(), deinterleaved[2].len());
    let chunked: Vec<[f32; 3]> = (0..deinterleaved[0].len()).fold(Vec::new(), |mut acc, it| {
        acc.push([deinterleaved[0][it], deinterleaved[1][it], deinterleaved[2][it]]);
        acc
    });

    assert_eq!(srgb, chunked);
    assert_eq!(slice.as_slice(), weave(deinterleaved).as_ref())
}

#[test]
fn nan_checks() {
    let it = [1e+3, -1e+3, 1e-3, -1e-3];
    // do these at f32 to faster approach bounds
    let fns: &[(&'static str, fn(&mut [f32; 3]))] = &[
        ("hsv_forwards", srgb_to_hsv),
        ("hsv_backwards", hsv_to_srgb),
        ("lrgb_forwards", srgb_to_lrgb),
        ("lrgb_backwards", lrgb_to_srgb),
        ("xyz_forwards", lrgb_to_xyz),
        ("xyz_backwards", xyz_to_lrgb),
        ("lab_forwards", xyz_to_cielab),
        ("lab_backwards", cielab_to_xyz),
        ("lch_forwards", lab_to_lch),
        ("lch_backwards", lch_to_lab),
        ("oklab_forwards", xyz_to_oklab),
        ("oklab_backwards", oklab_to_xyz),
        //("jzazbz_forwards", xyz_to_jzazbz), // ugh
        ("jzazbz_backwards", jzazbz_to_xyz),
    ];
    for (label, func) in fns {
        for a in it.iter() {
            for b in it.iter() {
                for c in it.iter() {
                    let from: [f32; 3] = [*a, *b, *c];
                    let mut to = from;
                    func(&mut to);
                    if to.iter().any(|c| !c.is_finite()) {
                        panic!("{} : {:?} -> {:?}", label, from, to);
                    }
                }
            }
        }
    }
}

#[test]
fn hue_wrap() {
    let it = (-1000..=2000).step_by(50);
    for a in it.clone() {
        for b in it.clone() {
            for c in it.clone() {
                let (a, b, c) = (a as f64 / 10.0, b as f64 / 10.0, c as f64 / 10.0);
                // lch
                let mut pixel = [a, b, c];
                convert_space(Space::SRGB, Space::CIELCH, &mut pixel);
                assert!(pixel[2] <= 360.0, "lch H was {}", pixel[2]);
                assert!(pixel[2] >= 0.0, "lch H was {}", pixel[2]);
                // hsv
                let mut pixel = [a, b, c];
                convert_space(Space::SRGB, Space::HSV, &mut pixel);
                assert!(pixel[0] <= 1.0, "hsv H was {}", pixel[0]);
                assert!(pixel[0] >= 0.0, "hsv H was {}", pixel[0]);
            }
        }
    }
}

#[test]
fn space_strings() {
    for space in Space::ALL {
        assert_eq!(Ok(*space), Space::try_from(space.to_string().as_str()))
    }
}

/// ### Other Tests ### }}}

// ### Str2Col ### {{{
#[test]
fn str2col_base() {
    assert_eq!(str2col("0.2, 0.5, 0.6"), Some((Space::SRGB, [0.2f32, 0.5, 0.6])))
}

#[test]
fn str2col_base_tight() {
    assert_eq!(str2col("0.2,0.5,0.6"), Some((Space::SRGB, [0.2f32, 0.5, 0.6])))
}

#[test]
fn str2col_base_lop() {
    assert_eq!(str2col("0.2,0.5, 0.6"), Some((Space::SRGB, [0.2f32, 0.5, 0.6])))
}

#[test]
fn str2col_base_bare() {
    assert_eq!(str2col("0.2 0.5 0.6"), Some((Space::SRGB, [0.2f32, 0.5, 0.6])))
}

#[test]
fn str2col_base_bare_fat() {
    assert_eq!(str2col("  0.2   0.5     0.6 "), Some((Space::SRGB, [0.2f32, 0.5, 0.6])))
}

#[test]
fn str2col_base_paren() {
    assert_eq!(str2col("(0.2 0.5 0.6)"), Some((Space::SRGB, [0.2f32, 0.5, 0.6])))
}

#[test]
fn str2col_base_paren2() {
    assert_eq!(str2col("{ 0.2 : 0.5 : 0.6 }"), Some((Space::SRGB, [0.2f32, 0.5, 0.6])))
}

#[test]
fn str2col_base_none() {
    assert_eq!(str2col::<f32, 3>("  0.2   0.5     f"), None)
}

#[test]
fn str2col_base_none2() {
    assert_eq!(str2col::<f32, 3>("0.2*0.5 0.6"), None)
}

#[test]
fn str2col_base_paren_none() {
    assert_eq!(str2col::<f32, 3>("(0.2 0.5 0.6"), None)
}

#[test]
fn str2col_base_paren_none2() {
    assert_eq!(str2col::<f32, 3>("0.2 0.5 0.6}"), None)
}

#[test]
fn str2col_lch() {
    assert_eq!(
        str2col("lch(50, 30, 160)"),
        Some((Space::CIELCH, [50.0f32, 30.0, 160.0]))
    )
}

#[test]
fn str2col_lch_space() {
    assert_eq!(
        str2col("lch 50, 30, 160"),
        Some((Space::CIELCH, [50.0f32, 30.0, 160.0]))
    )
}

#[test]
fn str2col_lch_colon() {
    assert_eq!(str2col("lch:50:30:160"), Some((Space::CIELCH, [50.0f32, 30.0, 160.0])))
}

#[test]
fn str2col_lch_semicolon() {
    assert_eq!(str2col("lch;50;30;160"), Some((Space::CIELCH, [50.0f32, 30.0, 160.0])))
}

#[test]
fn str2col_lch_mixed() {
    assert_eq!(
        str2col("lch; (50,30,160)"),
        Some((Space::CIELCH, [50.0f32, 30.0, 160.0]))
    )
}

#[test]
fn str2col_lch_mixed2() {
    assert_eq!(
        str2col("lch(50; 30; 160)"),
        Some((Space::CIELCH, [50.0f32, 30.0, 160.0]))
    )
}

#[test]
fn str2col_lch_mixed3() {
    assert_eq!(
        str2col("lch   (50   30  160)"),
        Some((Space::CIELCH, [50.0f32, 30.0, 160.0]))
    )
}

#[test]
fn str2col_hex() {
    assert_eq!(str2col(HEX), Some((Space::SRGB, irgb_to_srgb::<f32, 3>(IRGB))))
}

#[test]
fn str2col_perc100() {
    assert_eq!(
        str2col("oklch 100% 100% 100%"),
        Some((
            Space::OKLCH,
            [
                Space::OKLCH.srgb_quant100()[0],
                Space::OKLCH.srgb_quant100()[1],
                360.0f32
            ]
        ))
    )
}

#[test]
fn str2col_perc50() {
    assert_eq!(
        str2col("oklch 50.0% 50% 50.0000%"),
        Some((
            Space::OKLCH,
            [
                (Space::OKLCH.srgb_quant0()[0] + Space::OKLCH.srgb_quant100()[0]) / 2.0,
                (Space::OKLCH.srgb_quant0()[1] + Space::OKLCH.srgb_quant100()[1]) / 2.0,
                180.0f32,
            ]
        ))
    )
}

#[test]
fn str2col_perc0() {
    assert_eq!(
        str2col("oklch 0% 0% 0%"),
        Some((
            Space::OKLCH,
            [Space::OKLCH.srgb_quant0()[0], Space::OKLCH.srgb_quant0()[1], 0.0f32]
        ))
    )
}

#[test]
fn str2col_perc_mix() {
    assert_eq!(
        str2col("oklab 0.5 100.000% 0%"),
        Some((
            Space::OKLAB,
            [0.5f32, Space::OKLAB.srgb_quant100()[1], Space::OKLAB.srgb_quant0()[2]]
        ))
    )
}

#[test]
fn str2col_perc_inval() {
    assert_eq!(str2col::<f32, 3>("oklab 0.5 100 % 0%"), None);
    assert_eq!(str2col::<f32, 3>("oklab 0.5% %100% 0%"), None);
    assert_eq!(str2col::<f32, 3>("oklab 0.5 100%% 0%"), None);
}

#[test]
fn str2col_alpha() {
    assert_eq!(
        str2col("srgb 0, 0.5, 0.75, 1.0"),
        Some((Space::SRGB, [0f32, 0.5, 0.75, 1.0]))
    );
    assert_eq!(
        str2col("srgb 0, 0.5, 0.75, 1.0"),
        Some((Space::SRGB, [0f32, 0.5, 0.75]))
    );
    assert_eq!(
        str2col("srgb 10%, 20%, 50%, 80%"),
        Some((Space::SRGB, [0.1f32, 0.2, 0.5, 0.8]))
    );
    assert_eq!(
        str2col("srgb 10%, 20%, 50%, 80%"),
        Some((Space::SRGB, [0.1f32, 0.2, 0.5]))
    );
    let mut will_nan = str2col::<f32, 4>("srgb 0, 0.5, 0.75").unwrap();
    if will_nan.1[3].is_nan() {
        will_nan.1[3] = 0.12345
    }
    assert_eq!(will_nan, (Space::SRGB, [0f32, 0.5, 0.75, 0.12345]));
}

#[test]
fn str2space_base() {
    let pix: [f64; 3] =
        str2space("oklch : 0.62792590, 0.25768453, 29.22319405", Space::SRGB).expect("STR2SPACE_BASE FAIL");
    let reference = [1.00000000, 0.00000000, 0.00000000];
    pix_cmp(&[pix], &[reference], 1e-3, &[]);
}

#[test]
fn str2space_hex() {
    let pix: [f64; 3] = str2space(" { #FF0000 } ", Space::OKLCH).expect("STR2SPACE_HEX FAIL");
    let reference = [0.62792590, 0.25768453, 29.22319405];
    pix_cmp(&[pix], &[reference], 1e-3, &[]);
}
// ### Str2Col ### }}}
