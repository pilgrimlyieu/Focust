use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, Default, TS)]
pub struct ShortTimes(Vec<NaiveTime>);

impl ShortTimes {
    pub fn new(mut times: Vec<NaiveTime>) -> Self {
        times.sort();
        times.dedup();
        ShortTimes(times)
    }

    pub fn earliest_after(&self, time: &NaiveTime) -> Option<NaiveTime> {
        self.0.iter().filter(|&&t| t > *time).min().cloned()
    }

    pub fn first(&self) -> Option<NaiveTime> {
        self.0.first().cloned()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct TimeRange {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

impl Default for TimeRange {
    fn default() -> Self {
        TimeRange {
            start: NaiveTime::MIN,
            end: NaiveTime::MIN,
        }
    }
}

impl TimeRange {
    pub fn contains(&self, time: &NaiveTime) -> bool {
        if (self.start == NaiveTime::MIN) && (self.end == NaiveTime::MIN) {
            // if both are 00:00, treat as full day
            true
        } else {
            self.start <= *time && *time <= self.end
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ShortTimes tests
    #[test]
    fn test_short_times_new_sorts_and_deduplicates() {
        let times = vec![
            NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(14, 0, 0).unwrap(), // duplicate
            NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
        ];

        let short_times = ShortTimes::new(times);

        // Validate sorting
        assert_eq!(short_times.0[0], NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        assert_eq!(short_times.0[1], NaiveTime::from_hms_opt(11, 0, 0).unwrap());
        assert_eq!(short_times.0[2], NaiveTime::from_hms_opt(14, 0, 0).unwrap());

        // Validate deduplication
        assert_eq!(short_times.0.len(), 3);
    }

    #[test]
    fn test_short_times_earliest_after_finds_next_time() {
        let times = vec![
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        ];
        let short_times = ShortTimes::new(times);

        let current = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let next = short_times.earliest_after(&current);

        assert_eq!(next, Some(NaiveTime::from_hms_opt(14, 0, 0).unwrap()));
    }

    #[test]
    fn test_short_times_earliest_after_returns_none_when_all_passed() {
        let times = vec![
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
        ];
        let short_times = ShortTimes::new(times);

        let current = NaiveTime::from_hms_opt(20, 0, 0).unwrap();
        let next = short_times.earliest_after(&current);

        assert_eq!(next, None);
    }

    #[test]
    fn test_short_times_first_returns_earliest_time() {
        let times = vec![
            NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        ];
        let short_times = ShortTimes::new(times);

        assert_eq!(
            short_times.first(),
            Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap())
        );
    }

    #[test]
    fn test_short_times_first_returns_none_when_empty() {
        let short_times = ShortTimes::new(vec![]);
        assert_eq!(short_times.first(), None);
    }

    // TimeRange tests
    #[test]
    fn test_time_range_contains_time_within_range() {
        let range = TimeRange {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        };

        let time = NaiveTime::from_hms_opt(12, 30, 0).unwrap();
        assert!(range.contains(&time));
    }

    #[test]
    fn test_time_range_contains_start_boundary() {
        let range = TimeRange {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        };

        let time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        assert!(range.contains(&time));
    }

    #[test]
    fn test_time_range_contains_end_boundary() {
        let range = TimeRange {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        };

        let time = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
        assert!(range.contains(&time));
    }

    #[test]
    fn test_time_range_does_not_contain_time_before_start() {
        let range = TimeRange {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        };

        let time = NaiveTime::from_hms_opt(8, 59, 59).unwrap();
        assert!(!range.contains(&time));
    }

    #[test]
    fn test_time_range_does_not_contain_time_after_end() {
        let range = TimeRange {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        };

        let time = NaiveTime::from_hms_opt(17, 0, 1).unwrap();
        assert!(!range.contains(&time));
    }

    #[test]
    fn test_time_range_default() {
        let range = TimeRange::default();

        assert_eq!(range.start, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        assert_eq!(range.end, NaiveTime::from_hms_opt(0, 0, 0).unwrap());

        // Validate that a time in the middle of the day is contained
        let noon = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        assert!(range.contains(&noon));
    }
}
