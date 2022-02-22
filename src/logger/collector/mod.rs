use super::data_type::{LogMessagesStorage, LogMessage};
use super::super::ic_util;

pub fn store_log_message(storage: &mut dyn LogMessagesStorage, message: String, max_message_length: &usize) {
    let time_nanos = ic_util::get_ic_time_nanos();
    let time_nanos = match storage.get_last_log_message_time() {
        None => time_nanos,
        Some(previous_time_nanos) => {
            if time_nanos <= previous_time_nanos {
                previous_time_nanos + 1
            } else {
                time_nanos
            }
        }
    };

    let message = validate_message(message, max_message_length);

    let log_message = LogMessage {
        timeNanos: time_nanos,
        message,
    };

    storage.store_log_message(log_message);
}

fn validate_message(message: String, max_length: &usize) -> String {
    if message.len() > *max_length {
        String::from(&message[0..*max_length])
    } else {
        message
    }
}

#[cfg(test)]
mod tests {
    use crate::logger::collector::validate_message;

    #[test]
    fn test_validate_message() {
        assert_eq!(validate_message(String::from("abcd"), &5), String::from("abcd"));
        assert_eq!(validate_message(String::from("abcd"), &4), String::from("abcd"));
        assert_eq!(validate_message(String::from("abcd"), &3), String::from("abc"));
    }
}