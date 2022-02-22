use super::super::api_type::{Nanos, LogMessageData};

pub type LogMessage = LogMessageData;

pub trait LogMessagesInfo {

    fn get_log_messages_count(&self) -> u32;

    fn get_first_log_message_time(&self) -> Option<Nanos>;

    fn get_last_log_message_time(&self) -> Option<Nanos>;

}

pub trait LogMessagesSupplier: LogMessagesInfo {

    fn get_log_messages(&self, from_time_nanos: &Option<Nanos>) -> Box<dyn Iterator<Item=&'_ LogMessage> + '_>;

    fn get_log_messages_reverse(&self, up_to_time_nanos: &Option<Nanos>) -> Box<dyn Iterator<Item=&'_ LogMessage> + '_>;
}


pub trait LogMessagesStorage: LogMessagesInfo {

    fn store_log_message(&mut self, log_message: LogMessage);

    fn set_max_messages_count(&mut self, new_max_messages_count: usize);
}