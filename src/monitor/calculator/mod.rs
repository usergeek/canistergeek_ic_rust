use super::super::api_type;
use super::data_type;
use chrono::prelude::*;
use num_traits::ToPrimitive;

mod day_iterator;

const HOURLY_MAX_DAYS: usize = 9;
const DAILY_MAX_DAYS: usize = 365;

pub fn get_canister_metrics<'a>(
    parameters: &api_type::GetMetricsParameters,
    data_supplier: &'a dyn data_type::DayDataInfoSupplier,
) -> Result<api_type::CanisterMetricsData<'a>, &'a str> {
    let date_from = parameters.dateFromMillis.0.to_u64().unwrap() as i64;
    let date_to = parameters.dateToMillis.0.to_u64().unwrap() as i64;

    let iterator = day_iterator::DayIterator::new_reverse(date_from, date_to)?;

    match parameters.granularity {
        api_type::MetricsGranularity::hourly => Ok(api_type::CanisterMetricsData::hourly(
            iterator
                .take(HOURLY_MAX_DAYS)
                .filter_map(|date| {
                    data_supplier
                        .get_day_data_info(&date.year(), &date.month(), &date.day())
                        .map(|data| api_type::HourlyMetricsData {
                            timeMillis: candid::Int::from(date.timestamp_millis()),
                            canisterCycles: data.get_canister_cycles_data(),
                            canisterHeapMemorySize: data.get_canister_heap_memory_size_data(),
                            canisterMemorySize: data.get_canister_memory_size_data(),
                            updateCalls: data.get_update_calls_data(),
                        })
                })
                .collect(),
        )),
        api_type::MetricsGranularity::daily => Ok(api_type::CanisterMetricsData::daily(
            iterator
                .take(DAILY_MAX_DAYS)
                .filter_map(|date| {
                    data_supplier
                        .get_day_data_info(&date.year(), &date.month(), &date.day())
                        .map(|data| api_type::DailyMetricsData {
                            timeMillis: candid::Int::from(date.timestamp_millis()),
                            canisterCycles: calculate_numeric_metrics_entity(
                                data.get_canister_cycles_data(),
                            ),
                            canisterHeapMemorySize: calculate_numeric_metrics_entity(
                                data.get_canister_heap_memory_size_data(),
                            ),
                            canisterMemorySize: calculate_numeric_metrics_entity(
                                data.get_canister_memory_size_data(),
                            ),
                            updateCalls: data.get_update_calls_data().iter().sum(),
                        })
                })
                .collect(),
        )),
    }
}

fn calculate_numeric_metrics_entity(arr: &Vec<u64>) -> api_type::NumericEntity {
    let array_size = arr.len();

    let mut sum_for_avg: u64 = 0;
    let mut count_for_avg: u64 = 0;
    let mut min: u64 = 0;
    let mut max: u64 = 0;
    let mut first: u64 = 0;
    let mut last: u64 = 0;
    let mut avg: u64 = 0;

    if array_size > 0 {
        first = arr[0];
        last = arr[array_size - 1]
    };

    for value in arr.iter() {
        if *value > max {
            max = *value;
        }

        if *value > 0 {
            if min == 0 || *value < min {
                min = *value;
            }

            sum_for_avg += *value;
            count_for_avg += 1;
        }
    }

    if count_for_avg > 0 {
        avg = sum_for_avg / count_for_avg;
    }

    api_type::NumericEntity {
        avg,
        first,
        last,
        max,
        min,
    }
}
