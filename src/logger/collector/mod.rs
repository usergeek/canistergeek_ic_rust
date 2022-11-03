use super::super::ic_util;
use super::data_type::{LogMessage, LogMessagesStorage};

pub fn store_log_message(
    storage: &mut dyn LogMessagesStorage,
    message: String,
    max_message_length: &usize,
) {
    let time_nanos = ic_util::get_ic_time_nanos();
    store_log_message_int(storage, message, max_message_length, time_nanos)
}

fn store_log_message_int(
    storage: &mut dyn LogMessagesStorage,
    message: String,
    max_message_length: &usize,
    time_nanos: u64,
) {
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
    use super::super::super::logger::collector::validate_message;
    use super::super::data_type::{LogMessage, LogMessagesStorage};
    use crate::logger::data_type::LogMessagesInfo;

    #[test]
    fn test_validate_message() {
        assert_eq!(
            validate_message(String::from("abcd"), &5),
            String::from("abcd")
        );
        assert_eq!(
            validate_message(String::from("abcd"), &4),
            String::from("abcd")
        );
        assert_eq!(
            validate_message(String::from("abcd"), &3),
            String::from("abc")
        );
    }

    #[test]
    fn test_shifting_message_time() {
        struct FakeStorage {
            last_time: u64,
            messages_count: u32,
        }

        impl LogMessagesStorage for FakeStorage {
            fn store_log_message(&mut self, log_message: LogMessage) {
                self.last_time = log_message.timeNanos;
                self.messages_count += 1;
            }

            fn set_max_messages_count(&mut self, _new_max_messages_count: usize) {
                panic!()
            }
        }

        impl LogMessagesInfo for FakeStorage {
            fn get_log_messages_count(&self) -> u32 {
                self.messages_count
            }

            fn get_first_log_message_time(&self) -> Option<u64> {
                panic!()
            }

            fn get_last_log_message_time(&self) -> Option<u64> {
                if self.messages_count == 0 {
                    None
                } else {
                    Some(self.last_time)
                }
            }
        }

        let mut storage = FakeStorage {
            last_time: 0,
            messages_count: 0,
        };

        super::store_log_message_int(&mut storage, String::from("message1"), &20, 23);
        assert_eq!(storage.messages_count, 1);
        assert_eq!(storage.last_time, 23);

        super::store_log_message_int(&mut storage, String::from("message2"), &20, 23);
        assert_eq!(storage.messages_count, 2);
        assert_eq!(storage.last_time, 24);

        super::store_log_message_int(&mut storage, String::from("message2"), &20, 21);
        assert_eq!(storage.messages_count, 3);
        assert_eq!(storage.last_time, 25);

        super::store_log_message_int(&mut storage, String::from("message2"), &20, 27);
        assert_eq!(storage.messages_count, 4);
        assert_eq!(storage.last_time, 27);
    }
}
