use ic_cdk::export::{candid::{CandidType, Deserialize}};

// number of update calls in each time interval for a specific day.
pub type DayUpdateCallsCountData = Vec<u64>;

// canister heap memory size in each time interval for a specific day.
pub type DayCanisterHeapMemorySizeData = Vec<u64>;

// canister memory size in each time interval for a specific day.
pub type DayCanisterMemorySizeData = Vec<u64>;

// canister available cycles in each time interval for a specific day.
pub type DayCanisterCyclesData = Vec<u64>;

// specific day data with all necessary metrics
#[derive(Debug, CandidType, Deserialize)]
pub struct DayData {
    update_calls_data: DayUpdateCallsCountData,
    canister_heap_memory_size_data: DayCanisterHeapMemorySizeData,
    canister_memory_size_data: DayCanisterMemorySizeData,
    canister_cycles_data: DayCanisterCyclesData,
}

impl DayData {
    pub fn new(cell_count: &usize) -> Self {
        Self {
            update_calls_data: create_empty_vector(cell_count),
            canister_heap_memory_size_data: create_empty_vector(cell_count),
            canister_memory_size_data: create_empty_vector(cell_count),
            canister_cycles_data: create_empty_vector(cell_count)
        }
    }

    pub fn store(&mut self, cell: &usize, update_calls: u64, canister_heap_memory_size: u64, canister_memory_size: u64, canister_cycles: u64) {
        self.update_calls_data[*cell] = update_calls;
        self.canister_heap_memory_size_data[*cell] = canister_heap_memory_size;
        self.canister_memory_size_data[*cell] = canister_memory_size;
        self.canister_cycles_data[*cell] = canister_cycles;
    }

    pub fn increment_update_calls(&mut self, cell: &usize) {
        self.update_calls_data[*cell] += 1;
    }
}

fn create_empty_vector(cell_count: &usize) -> Vec<u64> {
    let mut vec = Vec::with_capacity(*cell_count);
    for _ in 0..*cell_count {
        vec.push(0_u64);
    }
    vec
}

pub trait DayDataInfo {
    fn get_update_calls_data(&self) -> &DayUpdateCallsCountData;
    fn get_canister_heap_memory_size_data(&self) -> &DayCanisterHeapMemorySizeData;
    fn get_canister_memory_size_data(&self) -> &DayCanisterMemorySizeData;
    fn get_canister_cycles_data(&self) -> &DayCanisterCyclesData;
}

impl DayDataInfo for DayData {
    fn get_update_calls_data(&self) -> &DayUpdateCallsCountData {
        &self.update_calls_data
    }

    fn get_canister_heap_memory_size_data(&self) -> &DayCanisterHeapMemorySizeData {
        &self.canister_heap_memory_size_data
    }

    fn get_canister_memory_size_data(&self) -> &DayCanisterMemorySizeData {
        &self.canister_memory_size_data
    }

    fn get_canister_cycles_data(&self) -> &DayCanisterCyclesData {
        &self.canister_cycles_data
    }
}

pub trait DayDataInfoSupplier {
    fn get_day_data_info(&self, year: &i32, month: &u32, day: &u32) -> Option<&dyn DayDataInfo>;
}

pub trait DayDataStorage {
    fn get_day_data(&mut self, year: &i32, month: &u32, day: &u32) -> Option<&mut DayData>;
    fn store_day_data(&mut self, year: &i32, month: &u32, day: &u32, day_data: DayData);
}