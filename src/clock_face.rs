//! Local receiver clock. No network time service or audio state is involved.
pub struct ClockFace {
    pub time: String,
    pub date: String,
}
impl ClockFace {
    pub fn now() -> Self {
        Self::from_datetime(chrono::Local::now().fixed_offset())
    }
    fn from_datetime(date: chrono::DateTime<chrono::FixedOffset>) -> Self {
        Self {
            time: date.format("%H:%M").to_string(),
            date: date.format("%d.%m.%Y.").to_string(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_clock_keeps_the_given_offset_and_24_hour_date() {
        for (input, time, date) in [
            ("2026-11-01T18:07:59+01:00", "18:07", "01.11.2026."),
            ("2027-01-01T00:00:00+01:00", "00:00", "01.01.2027."),
            ("2026-10-25T02:30:00+02:00", "02:30", "25.10.2026."),
            ("2026-10-25T02:30:00+01:00", "02:30", "25.10.2026."),
        ] {
            let clock =
                ClockFace::from_datetime(chrono::DateTime::parse_from_rfc3339(input).unwrap());
            assert_eq!(clock.time, time);
            assert_eq!(clock.date, date);
        }
    }
}
