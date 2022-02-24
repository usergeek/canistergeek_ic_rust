# canistergeek_ic_rust

`canistergeek_ic_rust` is the open-source tool for Internet Computer to track your project canisters cycles and memory status and collect log messages.

`canistergeek_ic_rust` can be integrated into your canisters as rust library which exposes following modules:
- `canistergeek_ic_rust::monitor` - public module that collects the data for specific canister by 5 minutes intervals.
- `canistergeek_ic_rust::logger` - public module that collects log messages for specific canister.

`canistergeek_ic_rust` should be used together with [Canistergeek-IC-JS](https://github.com/usergeek/canistergeek-ic-js) - Javascript library that fetches the data from canisters, performs all necessary calculations and displays it on a webpage

#### Memory consumption

- `canistergeek_ic_rust::monitor` - stored data for cycles and memory consumes ~6.5Mb per year per canister (assuming data points every 5 minutes).
- `canistergeek_ic_rust::logger` - depends on the length of messages and their number. (There is an [issue](https://github.com/dfinity/cdk-rs/issues/212) with heap memory size after upgrade).

## Metrics

### Collecting the data

Data can be collected in two ways: automatically and manually

1. Manually by calling `collectCanisterMetrics` external public method
2. Automatically by calling `canistergeek_ic_rust::monitor::collec_metrics();` in "update" methods in your canister to guarantee desired "Collect metrics" frequency. In some cases you may want to collect metrics in every "update" method to get the full picture in realtime and see how "update" methods influence canister price and capacity.

#### Update calls

Monitor collects the number of canister update calls

#### Cycles

Monitor collects how many cycles left at particular time using `ic_cdk::api::canister_balance()`.

#### Memory

Monitor collects how many memory bytes the canister consumes at particular time using `ic_cdk::api::stable::stable64_size() * WASM_PAGE_SIZE + core::arch::wasm32::memory_size(0) * WASM_PAGE_SIZE`.

#### Heap Memory

Monitor collects how many heap memory bytes the canister consumes at particular time using `core::arch::wasm32::memory_size(0) * WASM_PAGE_SIZE`.

## Logger

### Collecting log messages

Log messages can be collected by calling `canistergeek_ic_rust::logger::log_message(message: String);` method in "update" methods in your canister.

#### Log messages

Logger collects time/message pairs with a maximum message length of 4096 characters.

Default number of messages (10000) can be overridden with corresponding method in realtime.

## Installation

In file `Cargo.toml` your project, add dependency on crate:
```toml
canistergeek_ic_rust = "0.2.2"
```

## Usage

### Logger

Implement public methods in the canister in order to query collected log messages

```rust

// CANISTER LOGGER

/// Returns collected log messages based on passed parameters. Called from browser.
/// 
#[ic_cdk_macros::query(name = "getCanisterLog")]
pub async fn get_canister_log(request: canistergeek_ic_rust::api_type::CanisterLogRequest) -> Option<canistergeek_ic_rust::api_type::CanisterLogResponse<'static>> {
    validate_caller();
    canistergeek_ic_rust::logger::get_canister_log(request)
}

fn validate_caller() -> () {
    // limit access here!
}
```

### Monitor

Implement public methods in the canister in order to query collected data and optionally force collecting the data

```rust

// CANISTER MONITORING

/// Returns collected data based on passed parameters. Called from browser.
/// 
#[ic_cdk_macros::query(name = "getCanisterMetrics")]
pub async fn get_canister_metrics(parameters: canistergeek_ic_rust::api_type::GetMetricsParameters) -> Option<canistergeek_ic_rust::api_type::CanisterMetrics<'static>> {
    validate_caller();
    canistergeek_ic_rust::monitor::get_metrics(&parameters)
}

/// Force collecting the data at current time.
/// Called from browser or any canister `update` method.
///
#[ic_cdk_macros::update(name = "collectCanisterMetrics")]
pub async fn collect_canister_metrics() -> () {
    validate_caller();
    canistergeek_ic_rust::monitor::collect_metrics();
}

fn validate_caller() -> () {
    // limit access here!
}
```

#### Adjust "update" methods

Call `canistergeek_ic_rust::monitor::collect_metrics()` method in all "update" methods in your canister in order to automatically collect all data.

### Add post/pre upgrade hooks

Implement pre/post upgrade hooks.
This step is necessary to save collected data between canister upgrades.

```rust
#[ic_cdk_macros::pre_upgrade]
fn pre_upgrade_function() {
    let monitor_stable_data = canistergeek_ic_rust::monitor::pre_upgrade_stable_data();
    let logger_stable_data = canistergeek_ic_rust::logger::pre_upgrade_stable_data();
    ic_cdk::storage::stable_save((monitor_stable_data, logger_stable_data));
}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade_function() {
    let stable_data: Result<(canistergeek_ic_rust::monitor::PostUpgradeStableData, canistergeek_ic_rust::logger::PostUpgradeStableData), String> = ic_cdk::storage::stable_restore();
    match stable_data {
        Ok((monitor_stable_data, logger_stable_data)) => {
            canistergeek_ic_rust::monitor::post_upgrade_stable_data(monitor_stable_data);
            canistergeek_ic_rust::logger::post_upgrade_stable_data(logger_stable_data);
        }
        Err(_) => {}
    }
}
```

### Add candid api declaration to `did` file

In your canister did file `your_canister.did`, add next declaration:

```candid
...

type UpdateCallsAggregatedData = vec nat64;
type NumericEntity = 
 record {
   avg: nat64;
   first: nat64;
   last: nat64;
   max: nat64;
   min: nat64;
 };
type Nanos = nat64;
type MetricsGranularity = 
 variant {
   daily;
   hourly;
 };
type LogMessagesData = 
 record {
   message: text;
   timeNanos: Nanos;
 };
type HourlyMetricsData = 
 record {
   canisterCycles: CanisterCyclesAggregatedData;
   canisterHeapMemorySize: CanisterHeapMemoryAggregatedData;
   canisterMemorySize: CanisterMemoryAggregatedData;
   timeMillis: int;
   updateCalls: UpdateCallsAggregatedData;
 };
type GetMetricsParameters = 
 record {
   dateFromMillis: nat;
   dateToMillis: nat;
   granularity: MetricsGranularity;
 };
type GetLogMessagesParameters = 
 record {
   count: nat32;
   filter: opt GetLogMessagesFilter;
   fromTimeNanos: opt Nanos;
 };
type GetLogMessagesFilter = 
 record {
   analyzeCount: nat32;
   messageContains: opt text;
   messageRegex: opt text;
 };
type GetLatestLogMessagesParameters = 
 record {
   count: nat32;
   filter: opt GetLogMessagesFilter;
   upToTimeNanos: opt Nanos;
 };
type DailyMetricsData = 
 record {
   canisterCycles: NumericEntity;
   canisterHeapMemorySize: NumericEntity;
   canisterMemorySize: NumericEntity;
   timeMillis: int;
   updateCalls: nat64;
 };
type CanisterMetricsData = 
 variant {
   daily: vec DailyMetricsData;
   hourly: vec HourlyMetricsData;
 };
type CanisterMetrics = record {data: CanisterMetricsData;};
type CanisterMemoryAggregatedData = vec nat64;
type CanisterLogResponse = 
 variant {
   messages: CanisterLogMessages;
   messagesInfo: CanisterLogMessagesInfo;
 };
type CanisterLogRequest = 
 variant {
   getLatestMessages: GetLatestLogMessagesParameters;
   getMessages: GetLogMessagesParameters;
   getMessagesInfo;
 };
type CanisterLogMessagesInfo = 
 record {
   count: nat32;
   features: vec opt CanisterLogFeature;
   firstTimeNanos: opt Nanos;
   lastTimeNanos: opt Nanos;
 };
type CanisterLogMessages = 
 record {
   data: vec LogMessagesData;
   lastAnalyzedMessageTimeNanos: opt Nanos;
 };
type CanisterLogFeature =
 variant {
   filterMessageByContains;
   filterMessageByRegex;
 };
type CanisterHeapMemoryAggregatedData = vec nat64;
type CanisterCyclesAggregatedData = vec nat64;
service : {
  ...
  collectCanisterMetrics: () -> ();
  getCanisterLog: (opt CanisterLogRequest) -> (opt CanisterLogResponse) query;
  getCanisterMetrics: (GetMetricsParameters) -> (opt CanisterMetrics) query;
}

```

### LIMIT ACCESS TO YOUR DATA

🔴🔴🔴 We highly recommend limiting access by checking caller principal 🔴🔴🔴


```rust
fn validate_caller() {
    match ic_cdk::export::Principal::from_text("hozae-racaq-aaaaa-aaaaa-c") {
        Ok(caller) if caller == ic_cdk::caller() => (),
        _ => ic_cdk::trap("Invalid caller")
    }
}
```

## Full Example

```rust
#[ic_cdk_macros::pre_upgrade]
fn pre_upgrade_function() {
    let monitor_stable_data = canistergeek_ic_rust::monitor::pre_upgrade_stable_data();
    let logger_stable_data = canistergeek_ic_rust::logger::pre_upgrade_stable_data();
    ic_cdk::storage::stable_save((monitor_stable_data, logger_stable_data));
}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade_function() {
    let stable_data: Result<(canistergeek_ic_rust::monitor::PostUpgradeStableData, canistergeek_ic_rust::logger::PostUpgradeStableData), String> = ic_cdk::storage::stable_restore();
    match stable_data {
        Ok((monitor_stable_data, logger_stable_data)) => {
            canistergeek_ic_rust::monitor::post_upgrade_stable_data(monitor_stable_data);
            canistergeek_ic_rust::logger::post_upgrade_stable_data(logger_stable_data);
        }
        Err(_) => {}
    }
}

#[ic_cdk_macros::query(name = "getCanisterMetrics")]
pub async fn get_canister_metrics(parameters: canistergeek_ic_rust::api_type::GetMetricsParameters) -> Option<canistergeek_ic_rust::api_type::CanisterMetrics<'static>> {
    validate_caller();
    canistergeek_ic_rust::monitor::get_metrics(&parameters)
}

#[ic_cdk_macros::update(name = "collectCanisterMetrics")]
pub async fn collect_canister_metrics() -> () {
    validate_caller();
    canistergeek_ic_rust::monitor::collect_metrics();
}

#[ic_cdk_macros::query(name = "getCanisterLog")]
pub async fn get_canister_log(request: canistergeek_ic_rust::api_type::CanisterLogRequest) -> Option<canistergeek_ic_rust::api_type::CanisterLogResponse<'static>> {
    validate_caller();
    canistergeek_ic_rust::logger::get_canister_log(request)
}

fn validate_caller() -> () {
    match ic_cdk::export::Principal::from_text("hozae-racaq-aaaaa-aaaaa-c") {
        Ok(caller) if caller == ic_cdk::caller() => (),
        _ => ic_cdk::trap("Invalid caller")
    }
}

#[ic_cdk_macros::update(name = "doThis")]
pub async fn do_this() -> () {
    canistergeek_ic_rust::monitor::collect_metrics();
    canistergeek_ic_rust::logger::log_message(String::from("do_this"));
    // rest part of the your method...
}
```
