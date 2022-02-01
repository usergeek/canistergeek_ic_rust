use chrono::Duration;
use chrono::prelude::*;

/// Calendar days iterator.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// let result = DayIterator::new_reverse(23_i64, 23_i64);
/// let mut iter = result.unwrap();
/// assert_eq!(iter.next().unwrap().timestamp(), 0_i64);
/// assert_eq!(iter.next(), None);
/// ```
pub struct DayIterator {
    from_day: Date<Utc>,
    day: DateTime<Utc>,
}

impl DayIterator {

    pub fn new_reverse(from_millis: i64, to_millis: i64) -> Result<DayIterator, &'static str> {
        if from_millis > to_millis {
            return Err("From date more to date");
        }

        Ok(DayIterator {
            from_day: Utc.timestamp_millis(from_millis).date(),
            day: Utc.timestamp_millis(to_millis).date().and_hms_opt(0,0,0).unwrap(),
        })
    }
}

impl Iterator for DayIterator {
    type Item = DateTime<Utc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.day.date() < self.from_day  {
            return None;
        } else {
            let next_day = self.day.clone();
            self.day = self.day - Duration::days(1);
            Some(next_day)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::calculator::day_iterator::DayIterator;
    use chrono::prelude::*;
    use chrono::Duration;


    #[test]
    fn wrong() {
        match DayIterator::new_reverse(3_i64, 2_i64) {
            Err(_) => (),
            _ => panic!()
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
        let from = Utc.ymd(2020, 12, 28).and_hms(23, 12, 13);
        let to = Utc.ymd(2021, 1, 3).and_hms(23, 12, 13);

        let result = DayIterator::new_reverse(from.timestamp_millis(), to.timestamp_millis());
        let mut iter = result.unwrap();

        assert_eq!(iter.next().unwrap(), to.date().and_hms(0,0,0));
        assert_eq!(iter.next().unwrap(), to.date().and_hms(0,0,0) - Duration::days(1));
        assert_eq!(iter.next().unwrap(), to.date().and_hms(0,0,0) - Duration::days(2));
        assert_eq!(iter.next().unwrap(), to.date().and_hms(0,0,0) - Duration::days(3));
        assert_eq!(iter.next().unwrap(), to.date().and_hms(0,0,0) - Duration::days(4));
        assert_eq!(iter.next().unwrap(), to.date().and_hms(0,0,0) - Duration::days(5));
        assert_eq!(iter.next().unwrap(), to.date().and_hms(0,0,0) - Duration::days(6));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn some_max() {
        let from = Utc.ymd(2020, 12, 28).and_hms(23, 12, 13);
        let to = Utc.ymd(2021, 1, 3).and_hms(23, 12, 13);

        let result = DayIterator::new_reverse(from.timestamp_millis(), to.timestamp_millis());

        let v : Vec<i64> = result.unwrap().take(3).map(|d| {d.timestamp()}).collect();
        assert_eq!(1609632000_i64, v[0]);
        assert_eq!(1609545600_i64, v[1]);
        assert_eq!(1609459200_i64, v[2]);
        assert_eq!(3, v.len());
    }
}