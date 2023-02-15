use {
    crate::geo::geo_utils::{MAX_LAT_INCL, MIN_LAT_INCL, MAX_LON_INCL, MIN_LON_INCL},
    std::{
        error::Error,
        fmt::{Display, Formatter, Result as FmtResult},
    },
};

#[derive(Debug)]
pub enum GeoError {
    InvalidLatitude(f64),
    InvalidLongitude(f64),
}

impl Display for GeoError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            GeoError::InvalidLatitude(lat) => {
                write!(f, "invalid latitude {lat}; must be between {MIN_LAT_INCL} and {MAX_LAT_INCL}")
            }
            GeoError::InvalidLongitude(lon) => {
                write!(f, "invalid longitude {lon}; must be between {MIN_LON_INCL} and {MAX_LON_INCL}")
            }
        }
    }
}

impl Error for GeoError {}
