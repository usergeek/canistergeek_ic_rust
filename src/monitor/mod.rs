pub mod data_type;
pub mod calculator;
pub mod store;
pub mod collector;

// use crate::api_type;
use super::api_type::{GetMetricsParameters, CanisterMetrics};
use super::ic_util;
use store::Storage;
use collector::CanisterInfo;

pub type PreUpgradeStableData<'a> = (&'a u8, &'a store::DayDataTable);
pub type PostUpgradeStableData = (u8, store::DayDataTable);

const VERSION: u8 = 1;

static mut STORAGE: Option<Storage> = None;

fn storage<'a>() -> &'a mut Storage {
    unsafe {
        if let Some(s) = &mut STORAGE {
            s
        } else {
            STORAGE = Some(Storage::new());
            storage()
        }
    }
}

pub fn pre_upgrade_stable_data<'a>() -> PreUpgradeStableData<'a> {
    (&VERSION, storage().get_day_data_table())
}

pub fn post_upgrade_stable_data((version, upgrade_data): PostUpgradeStableData) {
    if version != VERSION {
        ic_cdk::print(std::format!("Can not upgrade stable data. Unsupported version {}", version));
    } else {
        unsafe {
            STORAGE = Some(Storage::init(upgrade_data));
        }
    }
}

pub fn collect_metrics() {
    collector::collect_canister_metrics(storage(), ic_util::get_ic_time_nanos(), || {
        let heap_memory_size = ic_util::get_heap_memory_size();
        let memory_size = ic_util::get_stable_memory_size() + heap_memory_size;
        let cycles = ic_util::get_cycles();
        CanisterInfo { heap_memory_size, memory_size, cycles }
    });
}

pub fn get_metrics<'a>(parameters: &GetMetricsParameters) -> Option<CanisterMetrics<'a>> {
    match calculator::get_canister_metrics(&parameters, storage()) {
        Ok(data) => Some(CanisterMetrics { data }),
        Err(_) => None
    }
}


#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use ic_cdk::export::candid;
    use crate::api_type::CanisterMetricsData;
    use super::collector;
    use super::calculator;

    #[test]
    fn test_metrics() {
        let mut storage = super::store::Storage::new();

        let time_nanos = Utc.ymd(2022, 01, 28).and_hms(13, 0, 0).timestamp_nanos() as u64;

        collector::collect_canister_metrics(&mut storage, time_nanos, || {
            let heap_memory_size = 234000;
            let memory_size = 345000;
            let cycles = 8787;
            collector::CanisterInfo { heap_memory_size, memory_size, cycles }
        });

        let time_nanos = Utc.ymd(2022, 01, 28).and_hms(9, 0, 0).timestamp_nanos() as u64;

        collector::collect_canister_metrics(&mut storage, time_nanos, || {
            let heap_memory_size = 1234000;
            let memory_size = 1345000;
            let cycles = 18787;
            collector::CanisterInfo { heap_memory_size, memory_size, cycles }
        });


        let params = crate::api_type::GetMetricsParameters {
            granularity: crate::api_type::MetricsGranularity::hourly,
            dateFromMillis: candid::Nat::from(Utc.ymd(2022, 01, 28).and_hms(11, 11, 11).timestamp_millis() as u64),
            dateToMillis: candid::Nat::from(Utc.ymd(2022, 01, 28).and_hms(11, 11, 11).timestamp_millis() as u64),
        };

        let result = calculator::get_canister_metrics(&params, &storage);
        dbg!(&result);

        let vector = match result.unwrap() {
            CanisterMetricsData::hourly(vector) => vector,
            _ => panic!()
        };

        assert_eq!(vector.len(), 1);
        let hourly_data = vector.get(0).unwrap();
        assert_eq!(hourly_data.timeMillis, candid::Int::from(1643328000000_i64));

        let cell_count = 288;
        let cell_9_hour = 9 * 3600 / 300;
        let cell_13_hour = 13 * 3600 / 300;

        assert_eq!(hourly_data.canisterCycles.len(), cell_count);
        assert_eq!(hourly_data.canisterCycles.get(cell_9_hour).unwrap(), &18787_u64);
        assert_eq!(hourly_data.canisterCycles.get(cell_13_hour).unwrap(), &8787_u64);

        assert_eq!(hourly_data.canisterHeapMemorySize.len(), cell_count);
        assert_eq!(hourly_data.canisterHeapMemorySize.get(cell_9_hour).unwrap(), &1234000_u64);
        assert_eq!(hourly_data.canisterHeapMemorySize.get(cell_13_hour).unwrap(), &234000_u64);

        assert_eq!(hourly_data.canisterMemorySize.len(), cell_count);
        assert_eq!(hourly_data.canisterMemorySize.get(cell_9_hour).unwrap(), &1345000_u64);
        assert_eq!(hourly_data.canisterMemorySize.get(cell_13_hour).unwrap(), &345000_u64);

        assert_eq!(hourly_data.updateCalls.len(), cell_count);
        assert_eq!(hourly_data.updateCalls.get(cell_9_hour).unwrap(), &1_u64);
        assert_eq!(hourly_data.updateCalls.get(cell_13_hour).unwrap(), &1_u64);

        for i in 0..cell_count {
            if i != cell_9_hour && i != cell_13_hour {
                assert_eq!(hourly_data.canisterCycles.get(i).unwrap(), &0_u64);
                assert_eq!(hourly_data.canisterHeapMemorySize.get(i).unwrap(), &0_u64);
                assert_eq!(hourly_data.canisterMemorySize.get(i).unwrap(), &0_u64);
                assert_eq!(hourly_data.updateCalls.get(i).unwrap(), &0_u64);
            }
        }
    }
}