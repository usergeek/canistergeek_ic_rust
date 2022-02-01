

const WASM_PAGE_SIZE: u64 = 65536;

pub fn get_ic_time_nanos() -> i64 {
    #[cfg(target_arch = "wasm32")]
        {
            ic_cdk::api::time() as i64
        }
    #[cfg(not(target_arch = "wasm32"))]
        {
            0
        }
}

pub fn get_cycles() -> u64 {
    #[cfg(target_arch = "wasm32")]
        {
            ic_cdk::api::canister_balance()
        }
    #[cfg(not(target_arch = "wasm32"))]
        {
            0
        }
}

pub fn get_stable_memory_size() -> u64 {
    #[cfg(target_arch = "wasm32")]
        {
            (ic_cdk::api::stable::stable64_size() as u64) * WASM_PAGE_SIZE
        }
    #[cfg(not(target_arch = "wasm32"))]
        {
            0
        }
}

pub fn get_heap_memory_size() -> u64 {
    #[cfg(target_arch = "wasm32")]
        {
            (core::arch::wasm32::memory_size(0) as u64) * WASM_PAGE_SIZE
        }

    #[cfg(not(target_arch = "wasm32"))]
        {
            0
        }
}

