mod data_type;
mod store;
mod collector;
mod calculator;

use super::api_type::{CanisterLogRequest, CanisterLogResponse};
use data_type::LogMessagesStorage;


pub type LogMessageStorage = store::Storage;

pub type PreUpgradeStableData<'a> = (&'a u8, &'a LogMessageStorage);
pub type PostUpgradeStableData = (u8, LogMessageStorage);

const VERSION: u8 = 1;

const DEFAULT_MAX_LOG_MESSAGES_COUNT: usize = 10_000;
const DEFAULT_MAX_LOG_MESSAGE_LENGTH: usize = 4096;

static mut STORAGE: Option<LogMessageStorage> = None;

fn get_storage<'a>() -> &'a mut LogMessageStorage {
    unsafe {
        if let Some(s) = &mut STORAGE {
            s
        } else {
            STORAGE = Some(LogMessageStorage::new(DEFAULT_MAX_LOG_MESSAGES_COUNT));
            get_storage()
        }
    }
}


// API

pub fn pre_upgrade_stable_data<'a>() -> PreUpgradeStableData<'a> {
    (&VERSION, get_storage())
}

pub fn post_upgrade_stable_data(data: PostUpgradeStableData) {
    match data {
        (VERSION, log_message_storage) => {
            unsafe {
                STORAGE = Some(log_message_storage);
            }
        }
        _ => {
            ic_cdk::print(std::format!("Can not upgrade stable log messages data. Unsupported version {}", data.0));
        }
    }
}

pub fn set_max_messages_count<'a>(limit: u32) {
    get_storage().set_max_messages_count(limit as usize);
}

pub fn log_message(message: String) {
    collector::store_log_message(get_storage(), message, &DEFAULT_MAX_LOG_MESSAGE_LENGTH);
}

pub fn get_canister_log<'a>(request: &CanisterLogRequest) -> Option<CanisterLogResponse<'a>> {
    match request {
        CanisterLogRequest::getMessagesInfo => {
            Some(CanisterLogResponse::messagesInfo(calculator::get_log_messages_info(get_storage())))
        },
        CanisterLogRequest::getMessages(parameters) => {
            match calculator::get_log_messages(get_storage(), parameters) {
                None => None,
                Some(messages) => Some(CanisterLogResponse::messages(messages))
            }
        },
        CanisterLogRequest::getLatestMessages(parameters) => {
            match calculator::get_latest_log_messages(get_storage(), parameters) {
                None => None,
                Some(messages) => Some(CanisterLogResponse::messages(messages))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::logger::calculator;
    use crate::logger::collector;
    use crate::logger::store::Storage;
    use crate::api_type::{GetLatestLogMessagesParameters};
    use crate::logger::data_type::LogMessagesInfo;

    #[test]
    fn test_empty_log_messages() {
        let storage = Storage::new(4);

        let params = GetLatestLogMessagesParameters {
            count: 10,
            upToTimeNanos: None,
            filterRegex: None,
        };

        let result = calculator::get_latest_log_messages(&storage, &params);
        let messages = result.expect("must zero elements");
        assert_eq!(messages.data.len(), 0);
    }

    #[test]
    fn test_chunk_log_messages() {
        let mut storage = Storage::new(4);

        collector::store_log_message(&mut storage, String::from("1 message"), &10);
        collector::store_log_message(&mut storage, String::from("2 message"), &10);
        collector::store_log_message(&mut storage, String::from("3 message"), &10);
        collector::store_log_message(&mut storage, String::from("4 message"), &3);

        let params = GetLatestLogMessagesParameters {
            count: 10,
            upToTimeNanos: None,
            filterRegex: None,
        };

        let result = calculator::get_latest_log_messages(&storage, &params);
        let messages = result.expect("must zero elements").data;
        assert_eq!(messages.len(), 4);

        assert_eq!(messages.get(0).unwrap().message, "4 m");
        assert_eq!(messages.get(1).unwrap().message, "3 message");
        assert_eq!(messages.get(2).unwrap().message, "2 message");
        assert_eq!(messages.get(3).unwrap().message, "1 message");


        let params = GetLatestLogMessagesParameters {
            count: 2,
            upToTimeNanos: None,
            filterRegex: None,
        };

        let messages = calculator::get_latest_log_messages(&storage, &params).unwrap().data;
        assert_eq!(messages.len(), 2);

        assert_eq!(messages.get(0).unwrap().message, "4 m");
        assert_eq!(messages.get(1).unwrap().message, "3 message");


        let params = GetLatestLogMessagesParameters {
            count: 1,
            upToTimeNanos: Some(messages.get(1).unwrap().timeNanos),
            filterRegex: None,
        };

        let messages = calculator::get_latest_log_messages(&storage, &params).unwrap().data;
        assert_eq!(messages.len(), 1);

        assert_eq!(messages.get(0).unwrap().message, "2 message");
    }

    #[test]
    fn test_filter_log_messages() {
        let mut storage = Storage::new(4);

        collector::store_log_message(&mut storage, String::from("message 1"), &1024);
        collector::store_log_message(&mut storage, String::from("сообщение abc 2 "), &1024);
        collector::store_log_message(&mut storage, String::from("message abc 3"), &1024);
        collector::store_log_message(&mut storage, String::from("message 4"), &10);

        let params = GetLatestLogMessagesParameters {
            count: 1,
            upToTimeNanos: None,
            filterRegex: Some(String::from("abc")),
        };

        let messages = calculator::get_latest_log_messages(&storage, &params).unwrap().data;
        assert_eq!(messages.len(), 1);

        assert_eq!(messages.get(0).unwrap().message, "message abc 3");


        let params = GetLatestLogMessagesParameters {
            count: 10,
            upToTimeNanos: None,
            filterRegex: Some(String::from("abc")),
        };

        let messages = calculator::get_latest_log_messages(&storage, &params).unwrap().data;
        assert_eq!(messages.len(), 2);

        assert_eq!(messages.get(0).unwrap().message, "message abc 3");
        assert_eq!(messages.get(1).unwrap().message, "сообщение abc 2 ");


        let params = GetLatestLogMessagesParameters {
            count: 10,
            upToTimeNanos: Some(messages.get(0).unwrap().timeNanos),
            filterRegex: Some(String::from("abc")),
        };

        let messages = calculator::get_latest_log_messages(&storage, &params).unwrap().data;
        assert_eq!(messages.len(), 1);

        assert_eq!(messages.get(0).unwrap().message, "сообщение abc 2 ");
    }

    #[test]
    fn test_log_messages_info() {
        let mut storage = Storage::new(4);
        assert_eq!(storage.get_log_messages_count(), 0);

        collector::store_log_message(&mut storage, String::from("message 1"), &1024);
        assert_eq!(storage.get_log_messages_count(), 1);

        collector::store_log_message(&mut storage, String::from("сообщение abc 2 "), &1024);
        collector::store_log_message(&mut storage, String::from("message abc 3"), &1024);
        collector::store_log_message(&mut storage, String::from("message 4"), &10);
        collector::store_log_message(&mut storage, String::from("message 5"), &10);
        assert_eq!(storage.get_log_messages_count(), 4);
    }

}