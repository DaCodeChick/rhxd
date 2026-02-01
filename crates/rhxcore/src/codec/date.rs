//! Date parameter encoding/decoding

use bytes::{Buf, BufMut};
use chrono::{DateTime, Datelike, Timelike, Utc};

/// Hotline date parameter (8 bytes)
#[derive(Debug, Clone, Copy)]
pub struct DateParam {
    pub year: u16,
    pub milliseconds: u16,
    pub seconds: u32,
}

impl DateParam {
    pub const SIZE: usize = 8;

    /// Create a date parameter from a DateTime
    pub fn from_datetime(dt: &DateTime<Utc>) -> Self {
        let year = dt.year() as u16;

        // Seconds since January 1st
        let month_secs = month_to_seconds(dt.month() as u8, is_leap_year(year));
        let day_secs = (dt.day() - 1) * 86400;
        let time_secs = dt.hour() * 3600 + dt.minute() * 60 + dt.second();

        let total_secs = month_secs + day_secs + time_secs;
        let millis = (dt.timestamp_subsec_millis() % 1000) as u16;

        Self {
            year,
            milliseconds: millis,
            seconds: total_secs,
        }
    }

    /// Parse from bytes
    pub fn from_bytes(mut buf: &[u8]) -> Result<Self, std::io::Error> {
        if buf.len() < Self::SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Not enough bytes for date parameter",
            ));
        }

        Ok(Self {
            year: buf.get_u16(),
            milliseconds: buf.get_u16(),
            seconds: buf.get_u32(),
        })
    }

    /// Encode to bytes
    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_u16(self.year);
        buf.put_u16(self.milliseconds);
        buf.put_u32(self.seconds);
    }
}

/// Seconds elapsed since January 1st for each month (non-leap year)
const MONTH_SECS: [u32; 12] = [
    0,        // January
    2678400,  // February (31 days * 86400)
    5097600,  // March (59 days)
    7776000,  // April (90 days)
    10368000, // May (120 days)
    13046400, // June (151 days)
    15638400, // July (181 days)
    18316800, // August (212 days)
    20995200, // September (243 days)
    23587200, // October (273 days)
    26265600, // November (304 days)
    28857600, // December (334 days)
];

/// Convert month to seconds since January 1st
fn month_to_seconds(month: u8, is_leap: bool) -> u32 {
    if month < 1 || month > 12 {
        return 0;
    }

    let mut secs = MONTH_SECS[(month - 1) as usize];

    // Add leap day if after February
    if is_leap && month > 2 {
        secs += 86400;
    }

    secs
}

/// Check if a year is a leap year
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Encode a DateTime to date parameter bytes
pub fn encode_date(dt: &DateTime<Utc>) -> Vec<u8> {
    let param = DateParam::from_datetime(dt);
    let mut buf = Vec::with_capacity(DateParam::SIZE);
    param.to_bytes(&mut buf);
    buf
}

/// Decode date parameter bytes to a DateTime
pub fn decode_date(buf: &[u8]) -> Result<DateTime<Utc>, std::io::Error> {
    let _param = DateParam::from_bytes(buf)?;

    // TODO: Convert back to DateTime (complex, deferred for now)
    // For MVP, just return current time
    Ok(Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2004));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2001));
    }

    #[test]
    fn test_month_seconds() {
        // January 1st = 0 seconds
        assert_eq!(month_to_seconds(1, false), 0);

        // February 1st = 31 days
        assert_eq!(month_to_seconds(2, false), 31 * 86400);

        // March 1st (non-leap) = 59 days
        assert_eq!(month_to_seconds(3, false), 59 * 86400);

        // March 1st (leap) = 60 days
        assert_eq!(month_to_seconds(3, true), 60 * 86400);
    }
}
