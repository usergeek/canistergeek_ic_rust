//! # canistergeek_ic_rust
//!
//! `canistergeek_ic_rust` is the open-source tool for Internet Computer
//! to track your project canisters cycles and memory status.

pub mod api_type;
pub mod data_type;
pub mod calculator;
pub mod store;
pub mod collector;
pub mod monitor;

#[cfg(test)]
mod tests {

    use chrono::prelude::*;
    use ic_cdk::export::candid;
    use crate::api_type::CanisterMetricsData;

    #[test]
    fn test() {
        let mut storage = crate::store::Storage::new();

        let time_nanos = Utc.ymd(2022, 01, 28).and_hms(13, 0 ,0).timestamp_nanos();

        crate::collector::collect_canister_metrics(&mut storage, time_nanos, || {
            let heap_memory_size = 234000;
            let memory_size = 345000;
            let cycles = 8787;
            crate::collector::CanisterInfo { heap_memory_size, memory_size, cycles }
        });

        let time_nanos = Utc.ymd(2022, 01, 28).and_hms(9, 0 ,0).timestamp_nanos();

        crate::collector::collect_canister_metrics(&mut storage, time_nanos, || {
            let heap_memory_size = 1234000;
            let memory_size = 1345000;
            let cycles = 18787;
            crate::collector::CanisterInfo { heap_memory_size, memory_size, cycles }
        });


        let params = crate::api_type::GetMetricsParameters {
            granularity: crate::api_type::MetricsGranularity::hourly,
            dateFromMillis: candid::Nat::from( Utc.ymd(2022, 01, 28).and_hms(11,11,11).timestamp_millis() as u64),
            dateToMillis: candid::Nat::from(Utc.ymd(2022, 01, 28).and_hms(11,11,11).timestamp_millis() as u64),
        };

        let result = crate::calculator::get_canister_metrics(&params, &storage);
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