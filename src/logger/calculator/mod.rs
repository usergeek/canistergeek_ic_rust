use super::super::api_type::{Nanos, GetLogMessagesParameters, GetLatestLogMessagesParameters, CanisterLogMessages, LogMessageData, CanisterLogMessagesInfo};
use super::data_type::LogMessagesSupplier;
use regex::Regex;

const MAX_CHUNK_SIZE : usize = 1024;

pub fn get_log_messages_info(log_message_supplier: &dyn LogMessagesSupplier) -> CanisterLogMessagesInfo {
    match log_message_supplier.get_log_messages_count() {
        0 => CanisterLogMessagesInfo { count: 0, firstTimeNanos: None, lastTimeNanos: None},
        count => {
            CanisterLogMessagesInfo {
                count,
                firstTimeNanos: log_message_supplier.get_first_log_message_time(),
                lastTimeNanos: log_message_supplier.get_last_log_message_time()
            }
        }
    }
}

pub fn get_log_messages<'a>(log_message_supplier: &'a dyn LogMessagesSupplier, parameters: &GetLogMessagesParameters) -> Option<CanisterLogMessages<'a>> {
    iterate_log_messages_int(false, &parameters.fromTimeNanos,
                             parameters.count as usize,
                             &parameters.filterRegex, log_message_supplier)
}

pub fn get_latest_log_messages<'a>(log_message_supplier: &'a dyn LogMessagesSupplier, parameters: &GetLatestLogMessagesParameters) -> Option<CanisterLogMessages<'a>> {
    iterate_log_messages_int(true, &parameters.upToTimeNanos,
                             parameters.count as usize,
                             &parameters.filterRegex, log_message_supplier)
}

fn iterate_log_messages_int<'a>(reverse: bool, time: &Option<Nanos>, count: usize, filter_regex: &Option<String>, log_message_supplier: &'a dyn LogMessagesSupplier) -> Option<CanisterLogMessages<'a>> {
    if count == 0 || count > MAX_CHUNK_SIZE {
        return None;
    }

    let regex: Option<Regex> = build_regex(filter_regex);

    let mut iterator_box = if reverse {
        log_message_supplier.get_log_messages_reverse(time)
    } else {
        log_message_supplier.get_log_messages(time)
    };

    let iterator = iterator_box.as_mut();

    let mut data: Vec<&'a LogMessageData> = Vec::with_capacity(count);

    if regex.is_some() {
        let regex = regex.as_ref().unwrap();

        for message in iterator {
            if !regex.is_match(&message.message) {
                continue;
            }

            data.push(message);

            if data.len() >= count {
                break;
            }
        }
    } else {
        for message in iterator {
            data.push(message);

            if data.len() >= count {
                break;
            }
        }
    }

    Some(CanisterLogMessages { data })
}

fn build_regex(filter_regex: &Option<String>) -> Option<Regex> {
    match filter_regex {
        None => None,
        Some(text) => {
            match Regex::new(text) {
                Ok(regex) => Some(regex),
                Err(_) => {
                    return None;
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::logger::calculator::build_regex;

    #[test]
    fn test_regex() {
        let regex = build_regex(&Some(String::from("abc")));
        let regex = regex.as_ref().unwrap();

        assert_eq!(regex.is_match("mess abc sss"), true);
        assert_eq!(regex.is_match("aa abc bb"), true);
        assert_eq!(regex.is_match("aa ab bb"), false);
    }

}

