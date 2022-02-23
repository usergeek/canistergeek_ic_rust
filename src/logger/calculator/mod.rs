use super::super::api_type::{Nanos, GetLogMessagesParameters, GetLatestLogMessagesParameters, CanisterLogMessages, LogMessageData, CanisterLogMessagesInfo, GetLogMessagesFilter};
use super::data_type::{LogMessagesSupplier, LogMessage};

mod regex_filter;
mod contains_filter;

const MAX_CHUNK_SIZE : usize = 1024;

pub fn get_log_messages_info(log_message_supplier: &dyn LogMessagesSupplier) -> CanisterLogMessagesInfo {
    match log_message_supplier.get_log_messages_count() {
        0 => CanisterLogMessagesInfo { count: 0, firstTimeNanos: None, lastTimeNanos: None, features: vec![]},
        count => {
            CanisterLogMessagesInfo {
                count,
                firstTimeNanos: log_message_supplier.get_first_log_message_time(),
                lastTimeNanos: log_message_supplier.get_last_log_message_time(),
                features: vec![]
            }
        }
    }
}

pub fn get_log_messages<'a>(log_message_supplier: &'a dyn LogMessagesSupplier, parameters: GetLogMessagesParameters) -> Result<CanisterLogMessages<'a>, &'a str> {
    iterate_log_messages_int(false, &parameters.fromTimeNanos,
                             parameters.count as usize,
                             parameters.filter, log_message_supplier)
}

pub fn get_latest_log_messages<'a>(log_message_supplier: &'a dyn LogMessagesSupplier, parameters: GetLatestLogMessagesParameters) -> Result<CanisterLogMessages<'a>, &'a str> {
    iterate_log_messages_int(true, &parameters.upToTimeNanos,
                             parameters.count as usize,
                             parameters.filter, log_message_supplier)
}

fn iterate_log_messages_int<'a>(reverse: bool, time: &Option<Nanos>, count: usize, filter: Option<GetLogMessagesFilter>, log_message_supplier: &'a dyn LogMessagesSupplier) -> Result<CanisterLogMessages<'a>, &'a str> {
    if count == 0 || count > MAX_CHUNK_SIZE {
        return Err("Wrong count number");
    }

    let mut iterator_box = if reverse {
        log_message_supplier.get_log_messages_reverse(time)
    } else {
        log_message_supplier.get_log_messages(time)
    };

    let iterator = iterator_box.as_mut();

    let mut data: Vec<&'a LogMessageData> = Vec::with_capacity(count);
    let mut message_time_nanos : Option<Nanos> = None;

    match filter {
        Some(filter) => {
            let mut filter_trait = build_filter(filter)?;

            for message in iterator {
                if filter_trait.is_stop() {
                    break;
                }

                message_time_nanos = Some(message.timeNanos);

                if !filter_trait.check_match(&message) {
                    continue;
                }

                data.push(message);

                if data.len() >= count {
                    break;
                }
            }
        }
        None => {
            for message in iterator {
                message_time_nanos = Some(message.timeNanos);
                data.push(message);

                if data.len() >= count {
                    break;
                }
            }
        }
    }

    Ok(CanisterLogMessages { data, lastAnalyzedMessageTimeNanos: message_time_nanos })
}

trait Filter {
    fn check_match(&mut self, log_message: &LogMessage) -> bool;
    fn is_stop(&self) -> bool;
}

fn build_filter<'a>(filter: GetLogMessagesFilter) -> Result<Box<dyn Filter>, &'a str> {
    match &filter.messageRegex {
        Some(regex_text) => {
            let regex_filter = regex_filter::MessageRegexFilter::create(filter.analyzeCount as usize, regex_text).unwrap();
            Ok(Box::new(regex_filter))
        },
        None => {
            match &filter.messageContains {
                Some(contains_text) => {
                    let contains_filter = contains_filter::MessageContainsFilter::create(filter.analyzeCount as usize, contains_text).unwrap();
                    Ok(Box::new(contains_filter))
                },
                None => Err("Empty filter")
            }
        }
    }
}


