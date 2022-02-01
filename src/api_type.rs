use ic_cdk::export::{candid, candid::{CandidType, Deserialize}};

#[allow(non_snake_case)]
#[derive(Debug, CandidType, Deserialize)]
pub struct GetMetricsParameters {
    pub granularity: MetricsGranularity,
    pub dateFromMillis: Millis,
    pub dateToMillis: Millis,
}

#[allow(non_camel_case_types)]
#[derive(Debug, CandidType, Deserialize)]
pub enum MetricsGranularity {
    hourly,
    daily,
}

pub type Millis = candid::Nat;


#[derive(Debug, CandidType)]
pub struct CanisterMetrics<'a> {
    pub data: CanisterMetricsData<'a>,
}

#[allow(non_camel_case_types)]
#[derive(Debug, CandidType)]
pub enum CanisterMetricsData<'a> {
    daily(Vec<DailyMetricsData>),
    hourly(Vec<HourlyMetricsData<'a>>),
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType)]
pub struct DailyMetricsData {
    pub canisterCycles: NumericEntity,
    pub canisterHeapMemorySize: NumericEntity,
    pub canisterMemorySize: NumericEntity,
    pub timeMillis: candid::Int,
    pub updateCalls: u64,
}

#[derive(Debug, CandidType)]
pub struct NumericEntity {
    pub avg: u64,
    pub first: u64,
    pub last: u64,
    pub max: u64,
    pub min: u64,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType)]
pub struct HourlyMetricsData<'a> {
    pub timeMillis: candid::Int,
    pub canisterCycles: CanisterCyclesAggregatedData<'a>,
    pub canisterHeapMemorySize: CanisterHeapMemoryAggregatedData<'a>,
    pub canisterMemorySize: CanisterMemoryAggregatedData<'a>,
    pub updateCalls: UpdateCallsAggregatedData<'a>,
}

pub type CanisterCyclesAggregatedData<'a> = &'a Vec<u64>;
pub type CanisterMemoryAggregatedData<'a> = &'a Vec<u64>;
pub type CanisterHeapMemoryAggregatedData<'a> = &'a Vec<u64>;
pub type UpdateCallsAggregatedData<'a> = &'a Vec<u64>;


