use candid::{CandidType, Deserialize};
use serde::Serialize;

#[allow(non_snake_case)]
#[derive(Debug, CandidType, Deserialize)]
pub struct GetInformationRequest {
    pub version: bool,
    pub status: Option<StatusRequest>,
    pub metrics: Option<MetricsRequest>,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType)]
pub struct GetInformationResponse<'a> {
    pub version: Option<candid::Nat>,
    pub status: Option<StatusResponse>,
    pub metrics: Option<MetricsResponse<'a>>,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType, Deserialize)]
pub struct StatusRequest {
    pub cycles: bool,
    pub memory_size: bool,
    pub heap_memory_size: bool,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType)]
pub struct StatusResponse {
    pub cycles: Option<u64>,
    pub memory_size: Option<u64>,
    pub heap_memory_size: Option<u64>,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType, Deserialize)]
pub struct MetricsRequest {
    pub parameters: GetMetricsParameters,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType)]
pub struct MetricsResponse<'a> {
    pub metrics: Option<CanisterMetrics<'a>>,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType, Deserialize)]
pub struct UpdateInformationRequest {
    pub metrics: Option<CollectMetricsRequestType>,
}

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Debug, CandidType, Deserialize)]
pub enum CollectMetricsRequestType {
    normal, //bump updateCalls and set cycles/memory once per interval
    force,  //bump updateCalls and set cycles/memory
}

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

// LOG messages

pub type Nanos = u64;

#[allow(non_camel_case_types)]
#[derive(Debug, CandidType, Deserialize)]
pub enum CanisterLogRequest {
    getMessagesInfo,
    getMessages(GetLogMessagesParameters),
    getLatestMessages(GetLatestLogMessagesParameters),
}

#[allow(non_camel_case_types)]
#[derive(Debug, CandidType)]
pub enum CanisterLogResponse<'a> {
    messagesInfo(CanisterLogMessagesInfo),
    messages(CanisterLogMessages<'a>),
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType, Deserialize)]
pub struct GetLogMessagesFilter {
    pub analyzeCount: u32,
    pub messageContains: Option<String>,
    pub messageRegex: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType, Deserialize)]
pub struct GetLogMessagesParameters {
    pub count: u32,
    pub filter: Option<GetLogMessagesFilter>,
    pub fromTimeNanos: Option<Nanos>,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType, Deserialize)]
pub struct GetLatestLogMessagesParameters {
    pub count: u32,
    pub filter: Option<GetLogMessagesFilter>,
    pub upToTimeNanos: Option<Nanos>,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType)]
pub struct CanisterLogMessages<'a> {
    pub data: Vec<&'a LogMessageData>,
    pub lastAnalyzedMessageTimeNanos: Option<Nanos>,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType, Deserialize, Clone, Serialize)]
pub struct LogMessageData {
    pub timeNanos: Nanos,
    pub message: String,
}

#[allow(non_camel_case_types)]
#[derive(Debug, CandidType)]
pub enum CanisterLogFeature {
    filterMessageByContains,
    filterMessageByRegex,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType)]
pub struct CanisterLogMessagesInfo {
    pub count: u32,
    pub features: Vec<Option<CanisterLogFeature>>,
    pub firstTimeNanos: Option<Nanos>,
    pub lastTimeNanos: Option<Nanos>,
}
