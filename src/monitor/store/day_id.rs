/// composite key of the day: 8 bits - year, 4 bits - month, 8 bits - day
pub type DayId = u32;

const MINIMAL_VALID_YEAR: i32 = 2000;

pub fn to_day_id(year: &i32, month: &u32, day: &u32) -> Result<DayId, &'static str> {
    let year_index: i32 = *year - MINIMAL_VALID_YEAR;

    if year_index < 0 {
        return Err("year less minimum");
    }

    let mut day_id: u32 = (year_index as u32) & 0x000000FF;
    day_id = (day_id << 4) | (*month & 0xF);
    day_id = (day_id << 8) | (*day & 0xFF);
    Ok(day_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_day_id_to_year(day_id: DayId) -> i32 {
        MINIMAL_VALID_YEAR + (((day_id >> 12) & 0x000000FF) as i32)
    }

    fn from_day_id_to_month(day_id: DayId) -> u32 {
        (day_id >> 8) & 0x0000000F
    }

    fn from_day_id_to_day(day_id: DayId) -> u32 {
        day_id & 0x000000FF
    }

    #[test]
    fn wrong() {
        match to_day_id(&(MINIMAL_VALID_YEAR - 1), &1, &1) {
            Err(_) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn test() {
        let day_id = to_day_id(&2021, &12, &31).unwrap();
        assert_eq!(from_day_id_to_year(day_id), 2021);
        assert_eq!(from_day_id_to_month(day_id), 12);
        assert_eq!(from_day_id_to_day(day_id), 31);
    }
}
