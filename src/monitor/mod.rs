use super::api_type;
use super::store::Storage;
use super::collector;
use super::collector::CanisterInfo;
use super::calculator;

use std::collections::BTreeMap;
use crate::api_type::CanisterMetrics;

mod ic_util;

pub type PreUpgradeStableData<'a> = (&'a u8, &'a super::store::DayDataTable);
pub type PostUpgradeStableData = (u8, super::store::DayDataTable);

const VERSION : u8 = 1;

static mut STORAGE: Option<Storage> = None;

fn storage<'a>() -> &'a mut Storage {
    unsafe {
        if let Some(s) = &mut STORAGE {
            s
        } else {
            STORAGE = Some(Storage::init(BTreeMap::new()));
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

pub fn get_metrics<'a>(parameters: &api_type::GetMetricsParameters) -> Option<api_type::CanisterMetrics<'a>> {
    match calculator::get_canister_metrics(&parameters, storage()) {
        Ok(data) => Some(CanisterMetrics {data}),
        Err(_) => None
    }
}