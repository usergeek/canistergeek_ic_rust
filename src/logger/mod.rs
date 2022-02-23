mod data_type;
mod store;
mod collector;
mod calculator;

use super::api_type::{CanisterLogRequest, CanisterLogResponse, CanisterLogFeature, CanisterLogMessagesInfo};
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

pub fn get_canister_log<'a>(request: Option<CanisterLogRequest>) -> Option<CanisterLogResponse<'a>> {
    match request {
        Some(CanisterLogRequest::getMessagesInfo) => {
            let info = calculator::get_log_messages_info(get_storage());
            let features = vec![
                Some(CanisterLogFeature::filterMessageByContains),
                Some(CanisterLogFeature::filterMessageByRegex)
            ];

            Some(CanisterLogResponse::messagesInfo(
                CanisterLogMessagesInfo {
                    features,
                    ..info
                }))
        }
        Some(CanisterLogRequest::getMessages(parameters)) => {
            match calculator::get_log_messages(get_storage(), parameters) {
                Err(_) => None,
                Ok(messages) => Some(CanisterLogResponse::messages(messages))
            }
        }
        Some(CanisterLogRequest::getLatestMessages(parameters)) => {
            match calculator::get_latest_log_messages(get_storage(), parameters) {
                Err(_) => None,
                Ok(messages) => Some(CanisterLogResponse::messages(messages))
            }
        }
        None => None
    }
}

#[cfg(test)]
mod tests {
    use super::super::logger::calculator;
    use super::super::logger::collector;
    use super::super::logger::store::Storage;
    use super::super::api_type::{GetLogMessagesParameters, GetLatestLogMessagesParameters, GetLogMessagesFilter};
    use super::super::logger::data_type::LogMessagesInfo;

    #[test]
    fn test_empty_log_messages() {
        let storage = Storage::new(4);

        let params = GetLatestLogMessagesParameters {
            count: 10,
            filter: None,
            upToTimeNanos: None,
        };

        let result = calculator::get_latest_log_messages(&storage, params);
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
            filter: None,
            upToTimeNanos: None,
        };

        let result = calculator::get_latest_log_messages(&storage, params).unwrap();
        let messages = result.data;
        assert_eq!(messages.len(), 4);
        assert_eq!(result.lastAnalyzedMessageTimeNanos.unwrap(), messages.get(3).unwrap().timeNanos);

        assert_eq!(messages.get(0).unwrap().message, "4 m");
        assert_eq!(messages.get(1).unwrap().message, "3 message");
        assert_eq!(messages.get(2).unwrap().message, "2 message");
        assert_eq!(messages.get(3).unwrap().message, "1 message");


        let params = GetLatestLogMessagesParameters {
            count: 2,
            filter: None,
            upToTimeNanos: None,
        };

        let result = calculator::get_latest_log_messages(&storage, params).unwrap();
        let messages = result.data;
        assert_eq!(messages.len(), 2);
        assert_eq!(result.lastAnalyzedMessageTimeNanos.unwrap(), messages.get(1).unwrap().timeNanos);

        assert_eq!(messages.get(0).unwrap().message, "4 m");
        assert_eq!(messages.get(1).unwrap().message, "3 message");


        let params = GetLatestLogMessagesParameters {
            count: 1,
            filter: None,
            upToTimeNanos: Some(messages.get(1).unwrap().timeNanos),
        };

        let result = calculator::get_latest_log_messages(&storage, params).unwrap();
        let messages = result.data;
        assert_eq!(messages.len(), 1);
        assert_eq!(result.lastAnalyzedMessageTimeNanos.unwrap(), messages.get(0).unwrap().timeNanos);

        assert_eq!(messages.get(0).unwrap().message, "2 message");
    }

    #[test]
    fn test_filter_log_messages_by_regex() {
        let mut storage = Storage::new(4);

        collector::store_log_message(&mut storage, String::from("message 1"), &1024);
        collector::store_log_message(&mut storage, String::from("сообщение abc 2 "), &1024);
        collector::store_log_message(&mut storage, String::from("message abc 3"), &1024);
        collector::store_log_message(&mut storage, String::from("message 4"), &10);

        let params = GetLogMessagesParameters {
            count: 4,
            filter: None,
            fromTimeNanos: None,
        };

        let result = calculator::get_log_messages(&storage, params).unwrap();
        let messages = result.data;
        assert_eq!(messages.len(), 4);
        assert_eq!(result.lastAnalyzedMessageTimeNanos.unwrap(), messages.get(3).unwrap().timeNanos);

        let message1 = messages.get(0).unwrap();
        let message2 = messages.get(1).unwrap();
        let message3 = messages.get(2).unwrap();
        let message4 = messages.get(3).unwrap();
        assert_eq!(message1.message, "message 1");
        assert_eq!(message2.message, "сообщение abc 2 ");
        assert_eq!(message3.message, "message abc 3");
        assert_eq!(message4.message, "message 4");


        let params = GetLatestLogMessagesParameters {
            count: 1,
            filter: Some(GetLogMessagesFilter {
                messageRegex: Some(String::from("abc")),
                messageContains: None,
                analyzeCount: 10,
            }),
            upToTimeNanos: None,
        };

        let result = calculator::get_latest_log_messages(&storage, params).unwrap();
        let messages = result.data;
        assert_eq!(messages.len(), 1);
        assert_eq!(result.lastAnalyzedMessageTimeNanos.unwrap(), messages.get(0).unwrap().timeNanos);

        assert_eq!(messages.get(0).unwrap().message, "message abc 3");


        let params = GetLatestLogMessagesParameters {
            count: 10,
            filter: Some(GetLogMessagesFilter {
                messageRegex: Some(String::from("abc")),
                messageContains: None,
                analyzeCount: 10,
            }),
            upToTimeNanos: None,
        };

        let result = calculator::get_latest_log_messages(&storage, params).unwrap();
        let messages = result.data;
        assert_eq!(messages.len(), 2);
        assert_eq!(result.lastAnalyzedMessageTimeNanos.unwrap(), message1.timeNanos);

        assert_eq!(messages.get(0).unwrap().message, "message abc 3");
        assert_eq!(messages.get(1).unwrap().message, "сообщение abc 2 ");


        let params = GetLatestLogMessagesParameters {
            count: 10,
            filter: Some(GetLogMessagesFilter {
                messageRegex: Some(String::from("abc")),
                messageContains: None,
                analyzeCount: 10,
            }),
            upToTimeNanos: Some(messages.get(0).unwrap().timeNanos),
        };

        let result = calculator::get_latest_log_messages(&storage, params).unwrap();
        let messages = result.data;
        assert_eq!(messages.len(), 1);
        assert_eq!(result.lastAnalyzedMessageTimeNanos.unwrap(), message1.timeNanos);

        assert_eq!(messages.get(0).unwrap().message, "сообщение abc 2 ");

        let params = GetLatestLogMessagesParameters {
            count: 10,
            filter: Some(GetLogMessagesFilter {
                messageRegex: Some(String::from("mess.*")),
                messageContains: Some(String::from("abC")),
                analyzeCount: 3,
            }),
            upToTimeNanos: None,
        };

        let result = calculator::get_latest_log_messages(&storage, params).unwrap();
        let messages = result.data;
        assert_eq!(messages.len(), 2);
        assert_eq!(result.lastAnalyzedMessageTimeNanos.unwrap(), message2.timeNanos);

        assert_eq!(messages.get(0).unwrap().message, "message 4");
        assert_eq!(messages.get(1).unwrap().message, "message abc 3");


        let params = GetLatestLogMessagesParameters {
            count: 10,
            filter: Some(GetLogMessagesFilter {
                messageRegex: None,
                messageContains: None,
                analyzeCount: 3,
            }),
            upToTimeNanos: None,
        };

        let result = calculator::get_latest_log_messages(&storage, params);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_filter_log_messages_by_contains() {
        let mut storage = Storage::new(4);

        collector::store_log_message(&mut storage, String::from("meSSage 1"), &1024);
        collector::store_log_message(&mut storage, String::from("сообщение Abc 2 "), &1024);
        collector::store_log_message(&mut storage, String::from("MEssage aBc 3"), &1024);
        collector::store_log_message(&mut storage, String::from("messaGE 4"), &10);

        let params = GetLogMessagesParameters {
            count: 4,
            filter: None,
            fromTimeNanos: None,
        };

        let result = calculator::get_log_messages(&storage, params).unwrap();
        let messages = result.data;
        assert_eq!(messages.len(), 4);
        assert_eq!(result.lastAnalyzedMessageTimeNanos.unwrap(), messages.get(3).unwrap().timeNanos);

        let message1 = messages.get(0).unwrap();
        let message2 = messages.get(1).unwrap();
        let message3 = messages.get(2).unwrap();
        let message4 = messages.get(3).unwrap();
        assert_eq!(message1.message, "meSSage 1");
        assert_eq!(message2.message, "сообщение Abc 2 ");
        assert_eq!(message3.message, "MEssage aBc 3");
        assert_eq!(message4.message, "messaGE 4");


        let params = GetLatestLogMessagesParameters {
            count: 1,
            filter: Some(GetLogMessagesFilter {
                messageRegex: None,
                messageContains: Some(String::from("abC")),
                analyzeCount: 10,
            }),
            upToTimeNanos: None,
        };

        let result = calculator::get_latest_log_messages(&storage, params).unwrap();
        let messages = result.data;
        assert_eq!(messages.len(), 1);
        assert_eq!(result.lastAnalyzedMessageTimeNanos.unwrap(), messages.get(0).unwrap().timeNanos);

        assert_eq!(messages.get(0).unwrap().message, message3.message);


        let params = GetLatestLogMessagesParameters {
            count: 10,
            filter: Some(GetLogMessagesFilter {
                messageRegex: None,
                messageContains: Some(String::from("abC")),
                analyzeCount: 10,
            }),
            upToTimeNanos: None,
        };

        let result = calculator::get_latest_log_messages(&storage, params).unwrap();
        let messages = result.data;
        assert_eq!(messages.len(), 2);
        assert_eq!(result.lastAnalyzedMessageTimeNanos.unwrap(), message1.timeNanos);

        assert_eq!(messages.get(0).unwrap().message, message3.message);
        assert_eq!(messages.get(1).unwrap().message, message2.message);


        let params = GetLatestLogMessagesParameters {
            count: 10,
            filter: Some(GetLogMessagesFilter {
                messageRegex: None,
                messageContains: Some(String::from("abC")),
                analyzeCount: 10,
            }),
            upToTimeNanos: Some(messages.get(0).unwrap().timeNanos),
        };

        let result = calculator::get_latest_log_messages(&storage, params).unwrap();
        let messages = result.data;
        assert_eq!(messages.len(), 1);
        assert_eq!(result.lastAnalyzedMessageTimeNanos.unwrap(), message1.timeNanos);

        assert_eq!(messages.get(0).unwrap().message, message2.message);
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