//! # canistergeek_ic_rust
//!
//! `canistergeek_ic_rust` is the open-source tool for Internet Computer
//! to track your project canisters cycles and memory status.

use crate::api_type::{
    CollectMetricsRequestType, GetInformationRequest, GetInformationResponse, MetricsResponse,
    UpdateInformationRequest,
};
use crate::monitor::{collect_metrics_int, get_metrics};

pub mod api_type;
pub mod ic_util;
pub mod logger;
pub mod monitor;

const API_VERSION: u8 = 1;

pub fn pre_upgrade_stable_data<'a>() -> (
    monitor::PreUpgradeStableData<'a>,
    logger::PreUpgradeStableData<'a>,
) {
    let monitor_stable_data = monitor::pre_upgrade_stable_data();
    let logger_stable_data = logger::pre_upgrade_stable_data();
    (monitor_stable_data, logger_stable_data)
}

pub fn post_upgrade_stable_data(
    (monitor_stable_data, logger_stable_data): (
        monitor::PostUpgradeStableData,
        logger::PostUpgradeStableData,
    ),
) {
    monitor::post_upgrade_stable_data(monitor_stable_data);
    logger::post_upgrade_stable_data(logger_stable_data);
}

pub fn update_information(request: UpdateInformationRequest) {
    if let Some(metrics_request) = request.metrics {
        match metrics_request {
            CollectMetricsRequestType::normal => collect_metrics_int(false),
            CollectMetricsRequestType::force => collect_metrics_int(true),
        };
    }
}

pub fn get_information<'a>(request: GetInformationRequest) -> GetInformationResponse<'a> {
    let version = if request.version {
        Some(candid::Nat::from(API_VERSION))
    } else {
        None
    };

    let status = request.status.map(monitor::get_status);
    let metrics = request.metrics.map(|request| MetricsResponse {
        metrics: get_metrics(&request.parameters),
    });
    let logs = logger::get_canister_log(request.logs);

    GetInformationResponse {
        version,
        status,
        metrics,
        logs,
    }
}
