# canistergeek_ic_rust

`canistergeek_ic_rust` is the open-source tool for Internet Computer to track your project canisters cycles and memory status.

`canistergeek_ic_rust` can be integrated into your canisters as rust library which exposes the `canistergeek_ic_rust::monitor` - public module that collects the data for specific canister by 5 minutes intervals.

`canistergeek_ic_rust` should be used together with [Canistergeek-IC-JS](https://github.com/usergeek/canistergeek-ic-js) - Javascript library that fetches the data from canisters, perform all necessary calculations and displays it on a webpage

Stored data consumes ~6.5Mb per year per canister (assuming data points every 5 minutes).

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


## Installation

In file `Cargo.toml` your project, add dependency on crate:
```toml
canistergeek_ic_rust = "0.1.1"
```

## Usage

Please perform the following steps

#### Add post/pre upgrade hooks

Implement pre/post upgrade hooks.
This step is necessary to save collected data between canister upgrades.

```rust
#[ic_cdk_macros::pre_upgrade]
fn pre_upgrade_function() {
    let stable_data = canistergeek_ic_rust::monitor::pre_upgrade_stable_data();
    ic_cdk::storage::stable_save(stable_data);
}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade_function() {
    let stable_data: Result<canistergeek_ic_rust::monitor::PostUpgradeStableData, String> = ic_cdk::storage::stable_restore();
    if stable_data.is_ok() {
        canistergeek_ic_rust::monitor::post_upgrade_stable_data(stable_data.unwrap());
    }
}
```

#### Implement public methods

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
#### Add candid api declaration to `did` file

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
type MetricsGranularity =
 variant {
   daily;
   hourly;
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
type CanisterHeapMemoryAggregatedData = vec nat64;
type CanisterCyclesAggregatedData = vec nat64;

service : {
    ...
    
  collectCanisterMetrics: () -> ();
  getCanisterMetrics: (GetMetricsParameters) -> (opt CanisterMetrics) query;
}
```


#### Adjust "update" methods

Call `canistergeek_ic_rust::monitor::collect_metrics()` method in all "update" methods in your canister in order to automatically collect all data.

#### LIMIT ACCESS TO YOUR DATA

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
    let stable_data = canistergeek_ic_rust::monitor::pre_upgrade_stable_data();
    ic_cdk::storage::stable_save(stable_data);
}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade_function() {
    let stable_data: Result<canistergeek_ic_rust::monitor::PostUpgradeStableData, String> = ic_cdk::storage::stable_restore();
    if stable_data.is_ok() {
        canistergeek_ic_rust::monitor::post_upgrade_stable_data(stable_data.unwrap());
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

fn validate_caller() -> () {
    match ic_cdk::export::Principal::from_text("hozae-racaq-aaaaa-aaaaa-c") {
        Ok(caller) if caller == ic_cdk::caller() => (),
        _ => ic_cdk::trap("Invalid caller")
    }
}

#[ic_cdk_macros::update(name = "doThis")]
pub async fn do_this() -> () {
    canistergeek_ic_rust::monitor::collect_metrics();
    // rest part of the your method...
}
```