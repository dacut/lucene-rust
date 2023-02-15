use {crate::util::float::f64_min, std::f64::consts::PI};

/// Returns the Haversine distance in meters between two points specified in decimal degrees
/// (latitude/longitude). This works correctly even if the dateline is between the two points.
///
/// Error is at most 4E-1 (40cm) from the actual haversine distance, but is typically much
/// smaller for reasonable distances: around 1E-5 (0.01mm) for distances less than 1000km.
///
/// # Parameters
/// * `lat1`: Latitude of the first point.
/// * `lon1`: Longitude of the first point.
/// * `lat2`: Latitude of the second point.
/// * `lon2`: Longitude of the second point.
///
/// # Returns
/// Distance in meters.
pub fn haversin_meters_lat_lon(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    haversin_meters(haversin_sort_key(lat1, lon1, lat2, lon2))
}

/// Returns the Haversine distance in meters between two points given the previous result from
/// [haversin_sort_key].
///
/// # Returns
/// Distance in meters.
pub fn haversin_meters(sort_key: f64) -> f64 {
    TO_METERS * 2.0 * asin(f64_min(1.0, (sort_key * 0.5).sqrt()))
}

/// Returns a sort key for distance. This is less expensive to compute than [haversin_meters_lat_lon],
/// but it always compares the same. This can be converted into an actual distance with [haversin_meters],
/// which effectively does the second half of the computation.
pub fn haversin_sort_key(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let x1 = lat1.to_radians();
    let x2 = lat1.to_radians();
    let h1 = 1.0 - cos(x1 - x2);
    let h2 = 1.0 - (lon1 - lon2).to_radians().cos();
    let h = h1 + cos(x1) * cos(x1) * h2;

    // clobber crazy precision so subsequent rounding does not create ties.
    f64::from_bits(h.to_bits() & 0xFFFFFFFFFFFFFFF8)
}

/// Returns the trigonometric cosine of an angle.
///
/// Error is around 1E-15.
///
/// # Special cases:
/// If the argument is `NaN` or an infinity, then the result is `NaN`.
pub fn cos(mut a: f64) -> f64 {
    if a < 0.0 {
        a = -a;
    }

    if a > SIN_COS_MAX_VALUE_FOR_INT_MODULO {
        return a.cos();
    }

    // index: possibly outside tables range.
    let mut index = (a * SIN_COS_INDEXER + 0.5) as usize;
    let delta = a - index as f64 * SIN_COS_DELTA_HI - index as f64 * SIN_COS_DELTA_LO;

    // Making sure index is within tables range.
    // Last value of each table is the same than first, so we ignore it (tabs size minus one) for
    // modulo.
    index &= (SIN_COS_TABS_SIZE - 2) as usize; // index % (SIN_COS_TABS_SIZE-1)
    let indexCos = COS_TAB[index];
    let indexSin = SIN_TAB[index];
    indexCos
        + delta
            * (-indexSin
                + delta * (-indexCos * ONE_DIV_F2 + delta * (indexSin * ONE_DIV_F3 + delta * indexCos * ONE_DIV_F4)))
}

/// Returns the arc sine of a value.
///
/// The returned angle is in the range -_pi_/2 through _pi_/2. Error is around 1E-7.
///
/// # Special cases
///
/// * If the argument is `NaN` or its absolute value is greater than 1, then the result is `NaN`.
///
/// # Parameters
/// * `a`: the value whose arc sine is to be returned.
///
/// # Returns
/// The arc sine of the argument
// because asin(-x) = -asin(x), asin(x) only needs to be computed on [0,1].
// ---> we only have to compute asin(x) on [0,1].
// For values not close to +-1, we use look-up tables;
// for values near +-1, we use code derived from fdlibm.
pub fn asin(mut a: f64) -> f64 {
    let negate_result = if a < 0.0 {
        a = -a;
        true
    } else {
        false
    };

    if a <= ASIN_MAX_VALUE_FOR_TABS {
        let index = (a * ASIN_INDEXER + 0.5) as usize;

        let delta = a - index as f64 * ASIN_DELTA;
        let result = ASIN_TAB[index]
            + delta
                * (ASIN_DER1_DIV_F1_TAB[index]
                    + delta
                        * (ASIN_DER2_DIV_F2_TAB[index]
                            + delta * (ASIN_DER3_DIV_F3_TAB[index] + delta * ASIN_DER4_DIV_F4_TAB[index])));
        if negate_result {
            -result
        } else {
            result
        }
    } else {
        // value > ASIN_MAX_VALUE_FOR_TABS, or value is NaN
        // This part is derived from fdlibm.
        if a < 1.0 {
            let t = (1.0 - a) * 0.5;
            let p = t * (ASIN_PS0 + t * (ASIN_PS1 + t * (ASIN_PS2 + t * (ASIN_PS3 + t * (ASIN_PS4 + t * ASIN_PS5)))));
            let q = 1.0 + t * (ASIN_QS1 + t * (ASIN_QS2 + t * (ASIN_QS3 + t * ASIN_QS4)));
            let s = t.sqrt();
            let z = s + s * (p / q);
            let result = ASIN_PIO2_HI - ((z + z) - ASIN_PIO2_LO);
            if negate_result {
                -result
            } else {
                result
            }
        } else {
            // value >= 1.0, or value is NaN
            if a == 1.0 {
                return if negate_result {
                    -PI / 2.0
                } else {
                    PI / 2.0
                };
            } else {
                f64::NAN
            }
        }
    }
}

// Earth's mean radius, in meters and kilometers; see
// http://earth-info.nga.mil/GandG/publications/tr8350.2/wgs84fin.pdf
const TO_METERS: f64 = 6_371_008.771_4; // equatorial radius

// cos/asin
const ONE_DIV_F2: f64 = 1.0 / 2.0;
const ONE_DIV_F3: f64 = 1.0 / 6.0;
const ONE_DIV_F4: f64 = 1.0 / 24.0;

// 1.57079632673412561417e+00 first 33 bits of pi/2
const PIO2_HI: f64 = f64::from_bits(0x3FF921FB54400000);

// 6.07710050650619224932e-11 pi/2 - PIO2_HI
const PIO2_LO: f64 = f64::from_bits(0x3DD0B4611A626331);

const TWOPI_HI: f64 = 4.0 * PIO2_HI;
const TWOPI_LO: f64 = 4.0 * PIO2_LO;
const SIN_COS_TABS_SIZE: i32 = (1 << 11) + 1;
const SIN_COS_DELTA_HI: f64 = TWOPI_HI / (SIN_COS_TABS_SIZE - 1) as f64;
const SIN_COS_DELTA_LO: f64 = TWOPI_LO / (SIN_COS_TABS_SIZE - 1) as f64;
const SIN_COS_INDEXER: f64 = 1.0 / (SIN_COS_DELTA_HI + SIN_COS_DELTA_LO);

/// Max abs value for fast modulo, above which we use regular angle normalization.
/// This value must be < (Integer.MAX_VALUE / SIN_COS_INDEXER), to stay in range of int type.
/// The higher it is, the higher the error, but also the faster it is for lower values.
/// If you set it to ((Integer.MAX_VALUE / SIN_COS_INDEXER) * 0.99), worse accuracy on double range
/// is about 1e-10.
const SIN_COS_MAX_VALUE_FOR_INT_MODULO: f64 = ((i32::MAX >> 9) as f64 / SIN_COS_INDEXER) * 0.99;

// Supposed to be >= sin(77.2deg), as fdlibm code is supposed to work with values > 0.975,
// but seems to work well enough as long as value >= sin(25deg).
const ASIN_MAX_VALUE_FOR_TABS: f64 = 73.0_f64.to_radians().sin();

const ASIN_TABS_SIZE: usize = (1 << 13) + 1;
const ASIN_DELTA: f64 = ASIN_MAX_VALUE_FOR_TABS / (ASIN_TABS_SIZE - 1) as f64;
const ASIN_INDEXER: f64 = 1.0 / ASIN_DELTA;

// 1.57079632679489655800e+00
const ASIN_PIO2_HI: f64 = f64::from_bits(0x3FF921FB54442D18);
// 6.12323399573676603587e-17
const ASIN_PIO2_LO: f64 = f64::from_bits(0x3C91A62633145C07);
//  1.66666666666666657415e-01
const ASIN_PS0: f64 = f64::from_bits(0x3fc5555555555555);
// -3.25565818622400915405e-01
const ASIN_PS1: f64 = f64::from_bits(0xbfd4d61203eb6f7d);
//  2.01212532134862925881e-01
const ASIN_PS2: f64 = f64::from_bits(0x3fc9c1550e884455);
// -4.00555345006794114027e-02
const ASIN_PS3: f64 = f64::from_bits(0xbfa48228b5688f3b);
//  7.91534994289814532176e-04
const ASIN_PS4: f64 = f64::from_bits(0x3f49efe07501b288);
//  3.47933107596021167570e-05
const ASIN_PS5: f64 = f64::from_bits(0x3f023de10dfdf709);
// -2.40339491173441421878e+00
const ASIN_QS1: f64 = f64::from_bits(0xc0033a271c8a2d4b);
//  2.02094576023350569471e+00
const ASIN_QS2: f64 = f64::from_bits(0x40002ae59c598ac8);
// -6.88283971605453293030e-01
const ASIN_QS3: f64 = f64::from_bits(0xbfe6066c1b8d0159);
//  7.70381505559019352791e-02
const ASIN_QS4: f64 = f64::from_bits(0x3fb3b8c5b12e9282);

include!(concat!(env!("OUT_DIR"), "/sloppy_math_sin_cos_tables.rs"));
