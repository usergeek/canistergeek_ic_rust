use chrono::prelude::*;

use crate::data_type::*;

const INTERVAL_IN_SECONDS: usize = 5 * 60;
const DAY_SECONDS : usize = 24 * 60 * 60;
const DAY_CELL_COUNT: usize = DAY_SECONDS / INTERVAL_IN_SECONDS;

pub struct CanisterInfo {
    pub heap_memory_size: u64,
    pub memory_size: u64,
    pub cycles: u64,
}

pub fn collect_canister_metrics<F>(storage: &mut dyn DayDataStorage, time_nanos: i64, canister_info_supplier: F)
where
    F: Fn() -> CanisterInfo
{
    let data_time = Utc.timestamp_nanos(time_nanos);

    match storage.get_day_data(&data_time.year(), &data_time.month(), &data_time.day()) {
        None => {
            let mut day_data = DayData::new(&DAY_CELL_COUNT);
            let cell = get_cell(&day_data, data_time);
            init_cell(&mut day_data, &cell, canister_info_supplier);
            storage.store_day_data(&data_time.year(), &data_time.month(), &data_time.day(), day_data);
        }
        Some(day_data) => {
            let cell = get_cell(day_data, data_time);
            if day_data.get_canister_cycles_data()[cell] == 0 {
                init_cell(day_data, &cell, canister_info_supplier);
            } else {
                day_data.increment_update_calls(&cell);
            }
        }
    }
}

fn init_cell<F>(day_data: &mut DayData, cell: &usize, canister_info_supplier: F)
where F: Fn() -> CanisterInfo {
    let canister_info = canister_info_supplier();
    day_data.store(&cell, 1,
                   canister_info.heap_memory_size,
                   canister_info.memory_size,
                   canister_info.cycles);
}

fn get_cell(day_data: & DayData, data_time: DateTime<Utc>) -> usize {
    let cell_count = day_data.get_update_calls_data().len() as usize;
    let seconds = (data_time.hour() * 3600 + data_time.minute() * 60 + data_time.second()) as usize;
    seconds / (DAY_SECONDS / cell_count)
}

