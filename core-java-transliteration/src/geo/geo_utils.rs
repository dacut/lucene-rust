use {crate::{geo::geo_error::GeoError, util::sloppy_math::haversin_meters}};

/// Minimum longitude value.
pub const MIN_LON_INCL: f64 = -180.0;

/// Maximum longitude value.
pub const MAX_LON_INCL: f64 = 180.0;

/// Minimum latitude value.
pub const MIN_LAT_INCL: f64 = -90.0;

/// Maximum latitude value.
pub const MAX_LAT_INCL: f64 = 90.0;

/// min longitude value in radians
pub const MIN_LON_RADIANS: f64 = MIN_LON_INCL.to_radians();

/// min latitude value in radians
pub const MIN_LAT_RADIANS: f64 = MIN_LAT_INCL.to_radians();

/// max longitude value in radians
pub const MAX_LON_RADIANS: f64 = MAX_LON_INCL.to_radians();

/// max latitude value in radians
pub const MAX_LAT_RADIANS: f64 = MAX_LAT_INCL.to_radians();

/// mean earth axis in meters
pub const EARTH_MEAN_RADIUS_METERS: f64 = 6_371_008.771_4;

/** validates latitude value is within standard +/-90 coordinate bounds */
pub fn check_latitude(latitude: f64) -> Result<(), GeoError> {
    if latitude.is_nan() || latitude < MIN_LAT_INCL || latitude > MAX_LAT_INCL {
        Err(GeoError::InvalidLatitude(latitude))
    } else {
        Ok(())
    }
}

/// binary search to find the exact sortKey needed to match the specified radius any sort key lte
/// this is a query match.
pub fn distance_query_sort_key(radius: f64) -> f64 {
    if radius >= haversin_meters(f64::MAX) {
        return haversin_meters(f64::MAX);
    }
  
    // this is a search through non-negative long space only
    let mut lo = 0;
    let mut hi: u64 = f64::MAX.to_bits();

    while lo <= hi {
        let mid = (lo + hi) >> 1;
        let sortKey = f64::from_bits(mid);
        let midRadius = haversin_meters(sortKey);
        if midRadius == radius {
            return sortKey;
        }
        
        if midRadius > radius {
            hi = mid - 1;
        } else {
            lo = mid + 1;
        }
    }
  
    // not found: this is because a user can supply an arbitrary radius, one that we will never
    // calculate exactly via our haversin method.
    let ceil = f64::from_bits(lo);
    assert!(haversin_meters(ceil) > radius);
    ceil
}

/// Returns a positive value if points a, b, and c are arranged in counter-clockwise order,
/// negative value if clockwise, zero if collinear.
pub fn orient(ax: f64, ay: f64, bx: f64, by: f64, cx: f64, cy: f64) -> i32 {
    let v1 = (bx - ax) * (cy - ay);
    let v2 = (cx - ax) * (by - ay);

    if v1 > v2 {
        1
    } else if v1 < v2 {
        -1
    } else {
        0
    }
}
