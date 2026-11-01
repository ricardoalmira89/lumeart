//! Las funciones matemáticas como las calcula Go en pxcli.
//!
//! goja no usa la libm de C: llama al paquete `math` de Go (código Go puro
//! en amd64, salvo `Exp` y `Log`, en ensamblador; `Exp` con FMA si el
//! procesador la tiene) y tiene su propio `hypot`. Sus resultados difieren
//! a veces en el último bit de los de QuickJS, y un dibujo que escribe
//! coordenadas sin redondear lo nota. Aquí están portadas operación a
//! operación, en el mismo orden, para que los SVG salgan idénticos.
//! Comprobadas con valores generados por Go
//! (`crates/lume-script/tests/fixtures/gomath.json`). Las usan los scripts
//! (como `Math` de goja) y la exportación a Godot (como el `math` de Go).

use std::f64::consts::PI;

const SHIFT: u32 = 52;
const MASK: u64 = 0x7FF;
const BIAS: i64 = 1023;

/// Constantes que Go calcula con precisión exacta antes de redondear.
const FOUR_OVER_PI: f64 = f64::from_bits(0x3FF4_5F30_6DC9_C883); // 4 / Pi
const PI_2: f64 = PI / 2.0; // exacto: dividir por 2
const PI_4: f64 = PI / 4.0;
const THREE_PI_4: f64 = f64::from_bits(0x4002_d97c_7f33_21d2); // 3 * Pi / 4

const fn bits(b: u64) -> f64 {
    f64::from_bits(b)
}

const SIN: [f64; 6] = [
    bits(0x3de5_d8fd_1fd1_9ccd),
    bits(0xbe5a_e5e5_a929_1f5d),
    bits(0x3ec7_1de3_567d_48a1),
    bits(0xbf2a_01a0_19bf_df03),
    bits(0x3f81_1111_1110_f7d0),
    bits(0xbfc5_5555_5555_5548),
];
const COS: [f64; 6] = [
    bits(0xbda8_fa49_a086_1a9b),
    bits(0x3e21_ee9d_7b4e_3f05),
    bits(0xbe92_7e4f_7eac_4bc6),
    bits(0x3efa_01a0_19c8_44f5),
    bits(0xbf56_c16c_16c1_4f91),
    bits(0x3fa5_5555_5555_554b),
];
const PI4A: f64 = bits(0x3fe9_21fb_4000_0000);
const PI4B: f64 = bits(0x3e64_442d_0000_0000);
const PI4C: f64 = bits(0x3ce8_4698_98cc_5170);
const REDUCE_THRESHOLD: f64 = (1u64 << 29) as f64;

const M_PI4: [u64; 20] = [
    0x0000_0000_0000_0001,
    0x45f3_06dc_9c88_2a53,
    0xf84e_afa3_ea69_bb81,
    0xb6c5_2b32_7887_2083,
    0xfca2_c757_bd77_8ac3,
    0x6e48_dc74_849b_a5c0,
    0x0c92_5dd4_13a3_2439,
    0xfc3b_d639_6253_4e7d,
    0xd104_6bea_5d76_8909,
    0xd338_e04d_68be_fc82,
    0x7323_ac73_06a6_73e9,
    0x3908_bf17_7bf2_5076,
    0x3ff1_2fff_bc0b_301f,
    0xde5e_2316_b414_da3e,
    0xda6c_fd9e_4f96_136e,
    0x9e8c_7ecd_3cbf_d45a,
    0xea4f_758f_d7cb_e2f6,
    0x7a0e_73ef_14a5_25d4,
    0xd7f6_bf62_3f1a_ba10,
    0xac06_608d_f8f6_d757,
];

/// `trigReduce` de Go (Payne-Hanek) para `x >= 2^29`.
fn trig_reduce(x: f64) -> (u64, f64) {
    if x < PI_4 {
        return (0, x);
    }
    let mut ix = x.to_bits();
    let exp = ((ix >> SHIFT) & MASK) as i64 - BIAS - i64::from(SHIFT);
    ix &= !(MASK << SHIFT);
    ix |= 1 << SHIFT;
    let digit = ((exp + 61) / 64) as usize;
    let bitshift = ((exp + 61) % 64) as u32;
    let join = |a: u64, b: u64| {
        // `b >> 64` en Go da 0.
        let low = if bitshift == 0 {
            0
        } else {
            b >> (64 - bitshift)
        };
        (a << bitshift) | low
    };
    let z0 = join(M_PI4[digit], M_PI4[digit + 1]);
    let z1 = join(M_PI4[digit + 1], M_PI4[digit + 2]);
    let z2 = join(M_PI4[digit + 2], M_PI4[digit + 3]);
    let z2hi = ((u128::from(z2) * u128::from(ix)) >> 64) as u64;
    let z1full = u128::from(z1) * u128::from(ix);
    let (z1hi, z1lo) = ((z1full >> 64) as u64, z1full as u64);
    let z0lo = z0.wrapping_mul(ix);
    let (lo, carry) = z1lo.overflowing_add(z2hi);
    let hi = z0lo.wrapping_add(z1hi).wrapping_add(u64::from(carry));
    let mut j = hi >> 61;
    let mut hi = (hi << 3) | (lo >> 61);
    let lz = hi.leading_zeros();
    let e = (BIAS as u64).wrapping_sub(u64::from(lz + 1));
    hi = (hi << (lz + 1)) | (lo >> (64 - (lz + 1)));
    hi >>= 64 - SHIFT;
    hi |= e << SHIFT;
    let mut z = f64::from_bits(hi);
    if j & 1 == 1 {
        j += 1;
        j &= 7;
        z -= 1.0;
    }
    (j, z * PI_4)
}

/// La reducción a un octante: `(j, z)`.
fn octant(x: f64) -> (u64, f64) {
    if x >= REDUCE_THRESHOLD {
        return trig_reduce(x);
    }
    let mut j = (x * FOUR_OVER_PI) as u64;
    let mut y = j as f64;
    if j & 1 == 1 {
        j += 1;
        y += 1.0;
    }
    j &= 7;
    (j, ((x - y * PI4A) - y * PI4B) - y * PI4C)
}

fn sin_poly(z: f64, zz: f64) -> f64 {
    z + z
        * zz
        * ((((((SIN[0] * zz) + SIN[1]) * zz + SIN[2]) * zz + SIN[3]) * zz + SIN[4]) * zz + SIN[5])
}

fn cos_poly(zz: f64) -> f64 {
    1.0 - 0.5 * zz
        + zz * zz
            * ((((((COS[0] * zz) + COS[1]) * zz + COS[2]) * zz + COS[3]) * zz + COS[4]) * zz
                + COS[5])
}

/// `math.Sin`.
pub fn sin(x: f64) -> f64 {
    if x == 0.0 || x.is_nan() {
        return x;
    }
    if x.is_infinite() {
        return f64::NAN;
    }
    let mut sign = x < 0.0;
    let (mut j, z) = octant(x.abs());
    if j > 3 {
        sign = !sign;
        j -= 4;
    }
    let zz = z * z;
    let y = if j == 1 || j == 2 {
        cos_poly(zz)
    } else {
        sin_poly(z, zz)
    };
    if sign { -y } else { y }
}

/// `math.Cos`.
pub fn cos(x: f64) -> f64 {
    if x.is_nan() || x.is_infinite() {
        return f64::NAN;
    }
    let mut sign = false;
    let (mut j, z) = octant(x.abs());
    if j > 3 {
        j -= 4;
        sign = !sign;
    }
    if j > 1 {
        sign = !sign;
    }
    let zz = z * z;
    let y = if j == 1 || j == 2 {
        sin_poly(z, zz)
    } else {
        cos_poly(zz)
    };
    if sign { -y } else { y }
}

/// `math.Tan`.
pub fn tan(x: f64) -> f64 {
    const P: [f64; 3] = [
        bits(0xc0c9_92d8_d24f_3f38),
        bits(0x4131_99ec_a5fc_9ddd),
        bits(0xc171_1fea_d329_9176),
    ];
    const Q: [f64; 5] = [
        1.0,
        bits(0x40ca_b8a5_eeb3_6572),
        bits(0xc134_27bc_582a_bc96),
        bits(0x4177_d98f_c2ea_d8ef),
        bits(0xc189_afe0_3cbe_5a31),
    ];
    if x == 0.0 || x.is_nan() {
        return x;
    }
    if x.is_infinite() {
        return f64::NAN;
    }
    let sign = x < 0.0;
    let x = x.abs();
    let (j, z) = if x >= REDUCE_THRESHOLD {
        trig_reduce(x)
    } else {
        let mut j = (x * FOUR_OVER_PI) as u64;
        let mut y = j as f64;
        if j & 1 == 1 {
            j += 1;
            y += 1.0;
        }
        (j, ((x - y * PI4A) - y * PI4B) - y * PI4C)
    };
    let zz = z * z;
    let mut y = if zz > 1e-14 {
        z + z
            * (zz * (((P[0] * zz) + P[1]) * zz + P[2])
                / ((((zz + Q[1]) * zz + Q[2]) * zz + Q[3]) * zz + Q[4]))
    } else {
        z
    };
    if j & 2 == 2 {
        y = -1.0 / y;
    }
    if sign { -y } else { y }
}

fn xatan(x: f64) -> f64 {
    const P0: f64 = bits(0xbfec_007f_a1f7_2594);
    const P1: f64 = bits(0xc030_2854_5b6b_807a);
    const P2: f64 = bits(0xc052_c08c_3688_0273);
    const P3: f64 = bits(0xc05e_b8bf_2d05_ba25);
    const P4: f64 = bits(0xc050_3669_fd28_ec8e);
    const Q0: f64 = bits(0x4038_dbc4_5b14_603c);
    const Q1: f64 = bits(0x4064_a0dd_43b8_fa25);
    const Q2: f64 = bits(0x407b_0e18_d2e2_be3b);
    const Q3: f64 = bits(0x407e_563f_13b0_49ea);
    const Q4: f64 = bits(0x4068_519e_fbbd_62ec);
    let z = x * x;
    let z = z * ((((P0 * z + P1) * z + P2) * z + P3) * z + P4)
        / (((((z + Q0) * z + Q1) * z + Q2) * z + Q3) * z + Q4);
    x * z + x
}

fn satan(x: f64) -> f64 {
    const MOREBITS: f64 = bits(0x3c91_a626_3314_5c07);
    const TAN3PIO8: f64 = bits(0x4003_504f_333f_9de6);
    if x <= 0.66 {
        return xatan(x);
    }
    if x > TAN3PIO8 {
        return PI_2 - xatan(1.0 / x) + MOREBITS;
    }
    PI_4 + xatan((x - 1.0) / (x + 1.0)) + 0.5 * MOREBITS
}

/// `math.Atan`.
pub fn atan(x: f64) -> f64 {
    if x == 0.0 {
        return x;
    }
    if x > 0.0 { satan(x) } else { -satan(-x) }
}

/// `math.Asin`.
pub fn asin(x: f64) -> f64 {
    if x == 0.0 {
        return x;
    }
    let sign = x < 0.0;
    let x = x.abs();
    if x > 1.0 {
        return f64::NAN;
    }
    let temp = (1.0 - x * x).sqrt();
    let temp = if x > 0.7 {
        PI_2 - satan(temp / x)
    } else {
        satan(x / temp)
    };
    if sign { -temp } else { temp }
}

/// `math.Acos`.
pub fn acos(x: f64) -> f64 {
    PI_2 - asin(x)
}

/// `math.Atan2`.
pub fn atan2(y: f64, x: f64) -> f64 {
    if y.is_nan() || x.is_nan() {
        return f64::NAN;
    }
    if y == 0.0 {
        if x >= 0.0 && !x.is_sign_negative() {
            return 0.0_f64.copysign(y);
        }
        return PI.copysign(y);
    }
    if x == 0.0 {
        return PI_2.copysign(y);
    }
    if x.is_infinite() {
        if x > 0.0 {
            return if y.is_infinite() {
                PI_4.copysign(y)
            } else {
                0.0_f64.copysign(y)
            };
        }
        return if y.is_infinite() {
            THREE_PI_4.copysign(y)
        } else {
            PI.copysign(y)
        };
    }
    if y.is_infinite() {
        return PI_2.copysign(y);
    }
    let q = atan(y / x);
    if x < 0.0 {
        if q <= 0.0 {
            return q + PI;
        }
        return q - PI;
    }
    q
}

/// `math.Exp` en amd64 (`exp_amd64.s`, con FMA).
pub fn exp(x: f64) -> f64 {
    const LOG2E: f64 = bits(0x3ff7_1547_652b_82fe);
    const LN2U: f64 = bits(0x3fe6_2e42_fefa_3000);
    const LN2L: f64 = bits(0x3d53_de6a_f278_ece6);
    const OVERFLOW: f64 = bits(0x4086_2e42_fefa_39ef);
    const C: [f64; 9] = [
        0.5,
        1.0,
        2.0,
        bits(0x3fc5_5555_5555_5555),
        bits(0x3fa5_5555_5555_5555),
        bits(0x3f81_1111_1111_1111),
        bits(0x3f56_c16c_16c1_6c17),
        bits(0x3f2a_01a0_1a01_a01a),
        bits(0x3efa_01a0_1a01_a01a),
    ];
    if x.is_nan() || x.is_infinite() {
        return if x == f64::NEG_INFINITY { 0.0 } else { x };
    }
    if x > OVERFLOW {
        return f64::INFINITY;
    }
    // CVTSD2SL: redondeo al par más cercano; fuera de rango, el mínimo.
    let k = (LOG2E * x).round_ties_even();
    let mut bx: i64 = if k >= f64::from(i32::MIN) && k <= f64::from(i32::MAX) {
        k as i64
    } else {
        i64::from(i32::MIN)
    };
    let kf = bx as f64;
    let mut x0 = (-kf).mul_add(LN2U, x);
    x0 = (-kf).mul_add(LN2L, x0);
    x0 *= 0.0625;
    let mut x1 = C[8];
    for c in [C[7], C[6], C[5], C[4], C[3], C[0], C[1]] {
        x1 = x0.mul_add(x1, c);
    }
    x0 *= x1;
    for _ in 0..3 {
        let t = x0 + C[2];
        x0 *= t;
    }
    let t = x0 + C[2];
    x0 = x0.mul_add(t, C[1]);
    // ldexp
    bx += 0x3FF;
    if bx <= 0 {
        if bx < -52 {
            return 0.0;
        }
        bx += 0x3FE;
        x0 *= f64::from_bits((bx as u64) << 52);
        bx = 1;
    } else if bx >= 0x7FF {
        return f64::INFINITY;
    }
    x0 * f64::from_bits((bx as u64) << 52)
}

/// `math.Log` en amd64 (`log_amd64.s`).
pub fn log(x: f64) -> f64 {
    const HSQRT2: f64 = bits(0x3fe6_a09e_667f_3bcd);
    const LN2HI: f64 = bits(0x3fe6_2e42_fee0_0000);
    const LN2LO: f64 = bits(0x3dea_39ef_3579_3c76);
    const L1: f64 = bits(0x3fe5_5555_5555_5593);
    const L2: f64 = bits(0x3fd9_9999_9997_fa04);
    const L3: f64 = bits(0x3fd2_4924_9422_9359);
    const L4: f64 = bits(0x3fcc_71c5_1d8e_78af);
    const L5: f64 = bits(0x3fc7_4664_96cb_03de);
    const L6: f64 = bits(0x3fc3_9a09_d078_c69f);
    const L7: f64 = bits(0x3fc2_f112_df3e_5244);
    let bits = x.to_bits();
    if bits & !(1 << 63) == 0 {
        return f64::NEG_INFINITY;
    }
    if (bits as i64) < 0 {
        return f64::NAN;
    }
    if bits >= 0x7FF0_0000_0000_0000 {
        return x;
    }
    let mut f1 = f64::from_bits((bits & 0x000F_FFFF_FFFF_FFFF) | 0x3FE0_0000_0000_0000);
    let mut k = (((bits >> 52) & 0x7FF) as i64 - 0x3FE) as f64;
    let small = HSQRT2 >= f1;
    let x3 = if small { 1.0 } else { 0.0 };
    k -= x3;
    f1 *= x3 + 1.0;
    let f = f1 - 1.0;
    let s = f / (2.0 + f);
    let s2 = s * s;
    let s4 = s2 * s2;
    let t1 = s2 * (((L7 * s4 + L5) * s4 + L3) * s4 + L1);
    let t2 = s4 * ((L6 * s4 + L4) * s4 + L2);
    let r = t1 + t2;
    let hfsq = 0.5 * f * f;
    let x3 = s * (r + hfsq) + LN2LO * k;
    let x0 = (hfsq - x3) - f;
    k * LN2HI - x0
}

/// `math.Frexp`.
fn frexp(f: f64) -> (f64, i64) {
    if f == 0.0 || f.is_infinite() || f.is_nan() {
        return (f, 0);
    }
    let (f, mut exp) = normalize(f);
    let mut x = f.to_bits();
    exp += ((x >> SHIFT) & MASK) as i64 - BIAS + 1;
    x &= !(MASK << SHIFT);
    x |= ((-1 + BIAS) as u64) << SHIFT;
    (f64::from_bits(x), exp)
}

fn normalize(x: f64) -> (f64, i64) {
    const SMALLEST_NORMAL: f64 = f64::MIN_POSITIVE;
    if x.abs() < SMALLEST_NORMAL {
        return (x * (1u64 << 52) as f64, -52);
    }
    (x, 0)
}

/// `math.Ldexp`.
fn ldexp(frac: f64, exp: i64) -> f64 {
    if frac == 0.0 || frac.is_infinite() || frac.is_nan() {
        return frac;
    }
    let (frac, e) = normalize(frac);
    let mut exp = exp + e;
    let mut x = frac.to_bits();
    exp += ((x >> SHIFT) & MASK) as i64 - BIAS;
    if exp < -1075 {
        return 0.0_f64.copysign(frac);
    }
    if exp > 1023 {
        return if frac < 0.0 {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    let mut m = 1.0;
    if exp < -1022 {
        exp += 53;
        m = 1.0 / (1u64 << 53) as f64;
    }
    x &= !(MASK << SHIFT);
    x |= ((exp + BIAS) as u64) << SHIFT;
    m * f64::from_bits(x)
}

/// `math.Modf`.
fn modf(f: f64) -> (f64, f64) {
    let integer = f.trunc();
    (integer, (f - integer).copysign(f))
}

fn is_odd_int(x: f64) -> bool {
    if x.abs() >= (1u64 << 53) as f64 {
        return false;
    }
    let (xi, xf) = modf(x);
    xf == 0.0 && (xi as i64) & 1 == 1
}

/// `math.Pow`.
fn go_pow(x: f64, y: f64) -> f64 {
    if y == 0.0 || x == 1.0 {
        return 1.0;
    }
    if y == 1.0 {
        return x;
    }
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    if x == 0.0 {
        if y < 0.0 {
            return if x.is_sign_negative() && is_odd_int(y) {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
        if y > 0.0 {
            return if x.is_sign_negative() && is_odd_int(y) {
                x
            } else {
                0.0
            };
        }
    } else if y.is_infinite() {
        if x == -1.0 {
            return 1.0;
        }
        return if (x.abs() < 1.0) == (y > 0.0) {
            0.0
        } else {
            f64::INFINITY
        };
    } else if x.is_infinite() {
        if x < 0.0 {
            return go_pow(1.0 / x, -y);
        }
        if y < 0.0 {
            return 0.0;
        }
        if y > 0.0 {
            return f64::INFINITY;
        }
    } else if y == 0.5 {
        return x.sqrt();
    } else if y == -0.5 {
        return 1.0 / x.sqrt();
    }
    let (mut yi, mut yf) = modf(y.abs());
    if yf != 0.0 && x < 0.0 {
        return f64::NAN;
    }
    // 1 << 63
    if yi >= 9_223_372_036_854_775_808.0 {
        if x == -1.0 {
            return 1.0;
        }
        return if (x.abs() < 1.0) == (y > 0.0) {
            0.0
        } else {
            f64::INFINITY
        };
    }
    let mut a1 = 1.0;
    let mut ae: i64 = 0;
    if yf != 0.0 {
        if yf > 0.5 {
            yf -= 1.0;
            yi += 1.0;
        }
        a1 = exp(yf * log(x));
    }
    let (mut x1, mut xe) = frexp(x);
    let mut i = yi as i64;
    while i != 0 {
        if !(-(1 << 12)..=(1 << 12)).contains(&xe) {
            ae += xe;
            break;
        }
        if i & 1 == 1 {
            a1 *= x1;
            ae += xe;
        }
        x1 *= x1;
        xe <<= 1;
        if x1 < 0.5 {
            x1 += x1;
            xe -= 1;
        }
        i >>= 1;
    }
    if y < 0.0 {
        a1 = 1.0 / a1;
        ae = -ae;
    }
    ldexp(a1, ae)
}

/// Potencia entera exacta, si cabe en 64 bits (`ipow` de goja); 0 si no.
fn ipow(base: i64, exp: i64) -> i64 {
    let mut result: i64 = 1;
    let mut b = base;
    let mut e = exp;
    while e > 0 {
        if e & 1 == 1 {
            match result.checked_mul(b) {
                Some(r) => result = r,
                None => return 0,
            }
        }
        e >>= 1;
        if e > 0 {
            match b.checked_mul(b) {
                Some(v) => b = v,
                None => return 0,
            }
        }
    }
    result
}

/// `Math.pow` de goja: potencias enteras exactas y, si no, `math.Pow`.
pub fn pow(x: f64, y: f64) -> f64 {
    let int = |v: f64| v.fract() == 0.0 && v.abs() < 9.007_199_254_740_992e15;
    if int(x) && int(y) && y >= 0.0 && !(x == 0.0 && x.is_sign_negative()) {
        if y == 0.0 {
            return 1.0;
        }
        if x == 0.0 {
            return 0.0;
        }
        let ip = ipow(x as i64, y as i64);
        if ip != 0 {
            return ip as f64;
        }
    }
    if x.abs() == 1.0 && y.is_infinite() {
        return f64::NAN;
    }
    if x == 1.0 && y.is_nan() {
        return f64::NAN;
    }
    go_pow(x, y)
}

/// `Math.hypot` de goja: suma de Kahan de los cuadrados normalizados.
pub fn hypot(values: &[f64]) -> f64 {
    let mut max: f64 = 0.0;
    let mut has_nan = false;
    let mut abs_values = Vec::with_capacity(values.len());
    for &v in values {
        if v.is_nan() {
            has_nan = true;
        } else {
            let a = v.abs();
            if a > max {
                max = a;
            }
            abs_values.push(a);
        }
    }
    if max == f64::INFINITY {
        return f64::INFINITY;
    }
    if has_nan {
        return f64::NAN;
    }
    if max == 0.0 {
        return 0.0;
    }
    let (mut sum, mut compensation) = (0.0_f64, 0.0_f64);
    for n in abs_values {
        let n = n / max;
        let summand = n * n - compensation;
        let preliminary = sum + summand;
        compensation = (preliminary - sum) - summand;
        sum = preliminary;
    }
    sum.sqrt() * max
}

/// `math.Hypot` de Go en amd64 (`hypot_amd64.s`): el mayor por
/// `sqrt(1 + (menor / mayor)²)`.
pub fn go_hypot(p: f64, q: f64) -> f64 {
    let (p, q) = (p.abs(), q.abs());
    if p.is_infinite() || q.is_infinite() {
        return f64::INFINITY;
    }
    if p.is_nan() || q.is_nan() {
        return f64::NAN;
    }
    if p == 0.0 && q == 0.0 {
        return 0.0;
    }
    let (big, small) = if p >= q { (p, q) } else { (q, p) };
    let r = small / big;
    big * (r * r + 1.0).sqrt()
}
