use chrono::prelude::*;
use chrono::Duration;

/// Calendar days iterator.
///
/// # Examples
///
/// Basic usage:
///
/// ```ignore
/// let result = DayIterator::new_reverse(23_i64, 23_i64);
/// let mut iter = result.unwrap();
/// assert_eq!(iter.next().unwrap().timestamp(), 0_i64);
/// assert_eq!(iter.next(), None);
/// ```
pub struct DayIterator {
    from_day: DateTime<Utc>,
    day: DateTime<Utc>,
}

impl DayIterator {
    pub fn new_reverse(from_millis: i64, to_millis: i64) -> Result<DayIterator, &'static str> {
        if from_millis > to_millis {
            return Err("From date more to date");
        }

        Ok(DayIterator {
            from_day: Utc.timestamp_millis_opt(from_millis).unwrap(),
            day: Utc
                .timestamp_millis_opt(to_millis).unwrap()
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
        })
    }
}

impl Iterator for DayIterator {
    type Item = DateTime<Utc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.day.date_naive() < self.from_day.date_naive() {
            None
        } else {
            let next_day = self.day;
            self.day -= Duration::days(1);
            Some(next_day)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DayIterator;
    use chrono::prelude::*;
    use chrono::Duration;

    #[test]
    fn wrong() {
        match DayIterator::new_reverse(3_i64, 2_i64) {
            Err(_) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn single() {
        let result = DayIterator::new_reverse(23_i64, 23_i64);
        let mut iter = result.unwrap();
        assert_eq!(iter.next().unwrap().timestamp(), 0_i64);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn some() {
        let from = Utc.with_ymd_and_hms(2020, 12, 28, 23, 12, 13).unwrap();
        let to = Utc.with_ymd_and_hms(2021, 1, 3, 23, 12, 13).unwrap();

        let result = DayIterator::new_reverse(from.timestamp_millis(), to.timestamp_millis());
        let mut iter = result.unwrap();

        assert_eq!(iter.next().unwrap(), to.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc());
        assert_eq!(
            iter.next().unwrap(),
            to.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc() - Duration::days(1)
        );
        assert_eq!(
            iter.next().unwrap(),
            to.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc() - Duration::days(2)
        );
        assert_eq!(
            iter.next().unwrap(),
            to.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc() - Duration::days(3)
        );
        assert_eq!(
            iter.next().unwrap(),
            to.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc() - Duration::days(4)
        );
        assert_eq!(
            iter.next().unwrap(),
            to.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc() - Duration::days(5)
        );
        assert_eq!(
            iter.next().unwrap(),
            to.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc() - Duration::days(6)
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn some_max() {
        let from = Utc.with_ymd_and_hms(2020, 12, 28, 23, 12, 13).unwrap();
        let to = Utc.with_ymd_and_hms(2021, 1, 3, 23, 12, 13).unwrap();

        let result = DayIterator::new_reverse(from.timestamp_millis(), to.timestamp_millis());

        let v: Vec<i64> = result.unwrap().take(3).map(|d| d.timestamp()).collect();
        assert_eq!(1609632000_i64, v[0]);
        assert_eq!(1609545600_i64, v[1]);
        assert_eq!(1609459200_i64, v[2]);
        assert_eq!(3, v.len());
    }
}
