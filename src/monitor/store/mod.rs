mod day_id;

use super::data_type::{DayData, DayDataInfo, DayDataInfoSupplier, DayDataStorage};
use day_id::DayId;
use std::collections::BTreeMap;

pub type DayDataTable = BTreeMap<DayId, DayData>;

#[derive(Default)]
pub struct Storage {
    day_data_table: DayDataTable,
}

impl Storage {
    pub fn init(day_data_table: DayDataTable) -> Self {
        Self { day_data_table }
    }

    pub fn get_day_data_table(&self) -> &DayDataTable {
        &self.day_data_table
    }
}

impl DayDataInfoSupplier for Storage {
    fn get_day_data_info(&self, year: &i32, month: &u32, day: &u32) -> Option<&dyn DayDataInfo> {
        match day_id::to_day_id(year, month, day) {
            Ok(day_id) => match self.day_data_table.get(&day_id) {
                None => None,
                Some(day_data) => Some(day_data),
            },
            _ => None,
        }
    }
}

impl DayDataStorage for Storage {
    fn get_day_data(&mut self, year: &i32, month: &u32, day: &u32) -> Option<&mut DayData> {
        let day_id = day_id::to_day_id(year, month, day).unwrap();
        self.day_data_table.get_mut(&day_id)
    }

    fn store_day_data(&mut self, year: &i32, month: &u32, day: &u32, day_data: DayData) {
        let day_id = day_id::to_day_id(year, month, day).unwrap();
        self.day_data_table.insert(day_id, day_data);
    }
}
