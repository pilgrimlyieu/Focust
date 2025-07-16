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
#[ts(rename_all = "camelCase")]
pub struct TimeRange {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

impl Default for TimeRange {
    fn default() -> Self {
        TimeRange {
            start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        }
    }
}

impl TimeRange {
    pub fn contains(&self, time: &NaiveTime) -> bool {
        self.start <= *time && *time <= self.end
    }
}
