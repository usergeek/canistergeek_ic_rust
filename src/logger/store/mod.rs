use super::super::api_type::Nanos;
use super::data_type::{LogMessage, LogMessagesInfo, LogMessagesStorage, LogMessagesSupplier};
use candid::CandidType;
use serde::{Deserialize, Serialize};

pub type LogMessageQueue = Vec<LogMessage>;

#[derive(Debug, CandidType, Deserialize, Serialize)]
pub struct Storage {
    queue: LogMessageQueue,
    max_count: usize,
    next: usize,
    full: bool,
}

impl Storage {
    pub fn new(max_messages_count: usize) -> Self {
        Storage::init(Vec::new(), max_messages_count, 0, false)
    }

    pub fn init(queue: LogMessageQueue, max_count: usize, next: usize, full: bool) -> Self {
        Self {
            queue,
            max_count,
            next,
            full,
        }
    }

    fn get_count(&self) -> usize {
        if self.full {
            self.max_count
        } else {
            self.next
        }
    }

    fn get_first_index(&self) -> usize {
        if self.full {
            self.next
        } else {
            0
        }
    }
}

impl LogMessagesInfo for Storage {
    fn get_log_messages_count(&self) -> u32 {
        self.queue.len() as u32
    }

    fn get_first_log_message_time(&self) -> Option<u64> {
        if self.queue.is_empty() {
            None
        } else {
            let index = if self.full { self.next } else { 0 };
            Some(self.queue[index].timeNanos)
        }
    }

    fn get_last_log_message_time(&self) -> Option<Nanos> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue[(self.max_count + self.next - 1) % self.max_count].timeNanos)
        }
    }
}

impl LogMessagesStorage for Storage {
    fn store_log_message(&mut self, log_message: LogMessage) {
        if self.full {
            self.queue[self.next] = log_message;
        } else {
            self.queue.push(log_message);
        }

        self.next += 1;

        if self.next == self.max_count {
            self.full = true;
            self.next = 0;
        }
    }

    fn set_max_messages_count(&mut self, new_max_messages_count: usize) {
        if self.max_count == new_max_messages_count {
            return;
        }

        let mut new_storage = Storage::new(new_max_messages_count);

        let count = self.get_count();
        let first_index = self.get_first_index();

        let range = if new_max_messages_count >= count {
            first_index..(first_index + count)
        } else {
            let start = first_index + count - new_max_messages_count;
            start..(start + new_max_messages_count)
        };

        for i in range {
            new_storage.store_log_message(self.queue.get(i % self.max_count).unwrap().clone());
        }

        self.queue = new_storage.queue;
        self.max_count = new_storage.max_count;
        self.next = new_storage.next;
        self.full = new_storage.full;
    }
}

impl LogMessagesSupplier for Storage {
    fn get_log_messages(
        &self,
        from_time_nanos: &Option<Nanos>,
    ) -> Box<dyn Iterator<Item = &'_ LogMessage> + '_> {
        Box::new(LogMessageIterator::create(self, from_time_nanos))
    }

    fn get_log_messages_reverse(
        &self,
        up_to_time_nanos: &Option<Nanos>,
    ) -> Box<dyn Iterator<Item = &'_ LogMessage> + '_> {
        Box::new(LogMessageIterator::create_reverse(self, up_to_time_nanos))
    }
}

struct LogMessageIterator<'a> {
    storage: &'a Storage,
    index: usize,
    delta: i32,
    next_count: usize,
}

impl<'a> LogMessageIterator<'a> {
    fn create(storage: &'a Storage, from_time_nanos: &Option<Nanos>) -> LogMessageIterator<'a> {
        if storage.queue.is_empty() {
            return LogMessageIterator {
                storage,
                index: 0,
                delta: 0,
                next_count: 0,
            };
        }

        let mut iterator = if storage.full {
            LogMessageIterator {
                storage,
                index: storage.next,
                delta: 1,
                next_count: storage.max_count,
            }
        } else {
            LogMessageIterator {
                storage,
                index: 0,
                delta: 1,
                next_count: storage.next,
            }
        };

        if from_time_nanos.is_some() {
            let from_time_nanos = from_time_nanos.unwrap();
            while !iterator.is_done()
                && iterator.get_current_message().unwrap().timeNanos <= from_time_nanos
            {
                iterator.shift_to_next();
            }
        }

        iterator
    }

    fn create_reverse(
        storage: &'a Storage,
        up_to_time_nanos: &Option<Nanos>,
    ) -> LogMessageIterator<'a> {
        if storage.queue.is_empty() {
            return LogMessageIterator {
                storage,
                index: 0,
                delta: 0,
                next_count: 0,
            };
        }

        let mut iterator = if storage.full {
            LogMessageIterator {
                storage,
                index: storage.max_count + storage.next - 1,
                delta: -1,
                next_count: storage.max_count,
            }
        } else {
            LogMessageIterator {
                storage,
                index: storage.next - 1,
                delta: -1,
                next_count: storage.next,
            }
        };

        if up_to_time_nanos.is_some() {
            let up_to_time_nanos = up_to_time_nanos.unwrap();
            while !iterator.is_done()
                && iterator.get_current_message().unwrap().timeNanos >= up_to_time_nanos
            {
                iterator.shift_to_next();
            }
        }

        iterator
    }

    fn get_current_message(&self) -> Option<&'a LogMessage> {
        self.storage.queue.get(self.index % self.storage.max_count)
    }

    fn is_done(&self) -> bool {
        self.next_count == 0
    }

    fn shift_to_next(&mut self) {
        self.index = (self.index as i32 + self.delta) as usize;
        self.next_count -= 1;
    }
}

impl<'a> Iterator for LogMessageIterator<'a> {
    type Item = &'a LogMessage;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done() {
            None
        } else {
            let message = self.get_current_message();
            self.shift_to_next();
            message
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::api_type::Nanos;
    use super::super::data_type::{
        LogMessage, LogMessagesInfo, LogMessagesStorage, LogMessagesSupplier,
    };
    use super::super::store::Storage;

    #[test]
    fn test_empty() {
        let storage = Storage::new(4);

        let mut iterator_box = storage.get_log_messages(&None);
        let iterator = iterator_box.as_mut();
        assert_eq!(iterator.next().is_none(), true);

        let mut iterator_box = storage.get_log_messages_reverse(&None);
        let iterator = iterator_box.as_mut();
        assert_eq!(iterator.next().is_none(), true);
    }

    #[test]
    fn test_cyclic() {
        let mut storage = Storage::new(4);

        {
            storage.store_log_message(LogMessage {
                timeNanos: 10,
                message: String::from("time 10"),
            });

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 10, "time 10");
            assert_eq!(iterator.next().is_none(), true);

            let mut iterator_box = storage.get_log_messages_reverse(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 10, "time 10");
            assert_eq!(iterator.next().is_none(), true);
        }

        {
            storage.store_log_message(LogMessage {
                timeNanos: 20,
                message: String::from("time 20"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 30,
                message: String::from("time 30"),
            });

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 10, "time 10");
            validate_message(iterator.next().unwrap(), 20, "time 20");
            validate_message(iterator.next().unwrap(), 30, "time 30");
            assert_eq!(iterator.next().is_none(), true);

            let mut iterator_box = storage.get_log_messages_reverse(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 30, "time 30");
            validate_message(iterator.next().unwrap(), 20, "time 20");
            validate_message(iterator.next().unwrap(), 10, "time 10");
            assert_eq!(iterator.next().is_none(), true);
        }

        {
            storage.store_log_message(LogMessage {
                timeNanos: 40,
                message: String::from("time 40"),
            });

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 10, "time 10");
            validate_message(iterator.next().unwrap(), 20, "time 20");
            validate_message(iterator.next().unwrap(), 30, "time 30");
            validate_message(iterator.next().unwrap(), 40, "time 40");
            assert_eq!(iterator.next().is_none(), true);

            let mut iterator_box = storage.get_log_messages_reverse(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 40, "time 40");
            validate_message(iterator.next().unwrap(), 30, "time 30");
            validate_message(iterator.next().unwrap(), 20, "time 20");
            validate_message(iterator.next().unwrap(), 10, "time 10");
            assert_eq!(iterator.next().is_none(), true);
        }

        {
            storage.store_log_message(LogMessage {
                timeNanos: 50,
                message: String::from("time 50"),
            });

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 20, "time 20");
            validate_message(iterator.next().unwrap(), 30, "time 30");
            validate_message(iterator.next().unwrap(), 40, "time 40");
            validate_message(iterator.next().unwrap(), 50, "time 50");
            assert_eq!(iterator.next().is_none(), true);

            let mut iterator_box = storage.get_log_messages_reverse(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 50, "time 50");
            validate_message(iterator.next().unwrap(), 40, "time 40");
            validate_message(iterator.next().unwrap(), 30, "time 30");
            validate_message(iterator.next().unwrap(), 20, "time 20");
            assert_eq!(iterator.next().is_none(), true);
        }
    }

    #[test]
    fn test_cyclic_with_from() {
        let mut storage = Storage::new(4);

        {
            storage.store_log_message(LogMessage {
                timeNanos: 10,
                message: String::from("time 10"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 20,
                message: String::from("time 20"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 30,
                message: String::from("time 30"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 40,
                message: String::from("time 40"),
            });

            let mut iterator_box = storage.get_log_messages(&Some(20));
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 30, "time 30");
            validate_message(iterator.next().unwrap(), 40, "time 40");
            assert_eq!(iterator.next().is_none(), true);

            let mut iterator_box = storage.get_log_messages_reverse(&Some(20));
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 10, "time 10");
            assert_eq!(iterator.next().is_none(), true);
        }
    }

    #[test]
    fn test_set_max_size() {
        let mut storage = Storage::new(2);

        {
            storage.set_max_messages_count(3);
            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            assert_eq!(iterator.next().is_none(), true);
        }

        {
            storage.store_log_message(LogMessage {
                timeNanos: 10,
                message: String::from("time 10"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 20,
                message: String::from("time 20"),
            });
            storage.set_max_messages_count(4);

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 10, "time 10");
            validate_message(iterator.next().unwrap(), 20, "time 20");
            assert_eq!(iterator.next().is_none(), true);
        }

        {
            storage.store_log_message(LogMessage {
                timeNanos: 30,
                message: String::from("time 30"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 40,
                message: String::from("time 40"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 50,
                message: String::from("time 50"),
            });
            storage.set_max_messages_count(5);

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 20, "time 20");
            validate_message(iterator.next().unwrap(), 30, "time 30");
            validate_message(iterator.next().unwrap(), 40, "time 40");
            validate_message(iterator.next().unwrap(), 50, "time 50");
            assert_eq!(iterator.next().is_none(), true);
        }

        {
            storage.store_log_message(LogMessage {
                timeNanos: 60,
                message: String::from("time 60"),
            });

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 20, "time 20");
            validate_message(iterator.next().unwrap(), 30, "time 30");
            validate_message(iterator.next().unwrap(), 40, "time 40");
            validate_message(iterator.next().unwrap(), 50, "time 50");
            validate_message(iterator.next().unwrap(), 60, "time 60");
            assert_eq!(iterator.next().is_none(), true);
        }

        {
            storage.set_max_messages_count(3);

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 40, "time 40");
            validate_message(iterator.next().unwrap(), 50, "time 50");
            validate_message(iterator.next().unwrap(), 60, "time 60");
            assert_eq!(iterator.next().is_none(), true);
        }

        {
            storage.set_max_messages_count(6);
            storage.set_max_messages_count(3);

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 40, "time 40");
            validate_message(iterator.next().unwrap(), 50, "time 50");
            validate_message(iterator.next().unwrap(), 60, "time 60");
            assert_eq!(iterator.next().is_none(), true);
        }
    }

    #[test]
    fn test_set_max_size_not_full_less() {
        let mut storage = Storage::new(4);
        storage.set_max_messages_count(5);

        {
            storage.store_log_message(LogMessage {
                timeNanos: 10,
                message: String::from("time 10"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 20,
                message: String::from("time 20"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 30,
                message: String::from("time 30"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 40,
                message: String::from("time 40"),
            });
            storage.set_max_messages_count(3);

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 20, "time 20");
            validate_message(iterator.next().unwrap(), 30, "time 30");
            validate_message(iterator.next().unwrap(), 40, "time 40");
            assert_eq!(iterator.next().is_none(), true);
        }
    }

    #[test]
    fn test_set_max_size_not_full_more() {
        let mut storage = Storage::new(5);

        {
            storage.store_log_message(LogMessage {
                timeNanos: 10,
                message: String::from("time 10"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 20,
                message: String::from("time 20"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 30,
                message: String::from("time 30"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 40,
                message: String::from("time 40"),
            });
            storage.set_max_messages_count(6);

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 10, "time 10");
            validate_message(iterator.next().unwrap(), 20, "time 20");
            validate_message(iterator.next().unwrap(), 30, "time 30");
            validate_message(iterator.next().unwrap(), 40, "time 40");
            assert_eq!(iterator.next().is_none(), true);
        }
    }

    #[test]
    fn test_set_max_size_full_less() {
        let mut storage = Storage::new(4);

        {
            storage.store_log_message(LogMessage {
                timeNanos: 10,
                message: String::from("time 10"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 20,
                message: String::from("time 20"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 30,
                message: String::from("time 30"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 40,
                message: String::from("time 40"),
            });
            storage.set_max_messages_count(3);

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 20, "time 20");
            validate_message(iterator.next().unwrap(), 30, "time 30");
            validate_message(iterator.next().unwrap(), 40, "time 40");
            assert_eq!(iterator.next().is_none(), true);
        }
    }

    #[test]
    fn test_set_max_size_full_more() {
        let mut storage = Storage::new(3);

        {
            storage.store_log_message(LogMessage {
                timeNanos: 10,
                message: String::from("time 10"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 20,
                message: String::from("time 20"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 30,
                message: String::from("time 30"),
            });
            storage.store_log_message(LogMessage {
                timeNanos: 40,
                message: String::from("time 40"),
            });
            storage.set_max_messages_count(5);

            let mut iterator_box = storage.get_log_messages(&None);
            let iterator = iterator_box.as_mut();
            validate_message(iterator.next().unwrap(), 20, "time 20");
            validate_message(iterator.next().unwrap(), 30, "time 30");
            validate_message(iterator.next().unwrap(), 40, "time 40");
            assert_eq!(iterator.next().is_none(), true);
        }
    }

    #[test]
    fn test_info() {
        let mut storage = Storage::new(2);
        assert_eq!(storage.get_log_messages_count(), 0);
        assert_eq!(storage.get_first_log_message_time().is_none(), true);
        assert_eq!(storage.get_last_log_message_time().is_none(), true);

        storage.store_log_message(LogMessage {
            timeNanos: 10,
            message: String::from("time 10"),
        });
        assert_eq!(storage.get_log_messages_count(), 1);
        assert_eq!(storage.get_first_log_message_time().unwrap(), 10_u64);
        assert_eq!(storage.get_last_log_message_time().unwrap(), 10_u64);

        storage.store_log_message(LogMessage {
            timeNanos: 20,
            message: String::from("time 20"),
        });
        assert_eq!(storage.get_log_messages_count(), 2);
        assert_eq!(storage.get_first_log_message_time().unwrap(), 10_u64);
        assert_eq!(storage.get_last_log_message_time().unwrap(), 20_u64);

        storage.store_log_message(LogMessage {
            timeNanos: 30,
            message: String::from("time 30"),
        });
        assert_eq!(storage.get_log_messages_count(), 2);
        assert_eq!(storage.get_first_log_message_time().unwrap(), 20_u64);
        assert_eq!(storage.get_last_log_message_time().unwrap(), 30_u64);

        storage.store_log_message(LogMessage {
            timeNanos: 40,
            message: String::from("time 40"),
        });
        assert_eq!(storage.get_log_messages_count(), 2);
        assert_eq!(storage.get_first_log_message_time().unwrap(), 30_u64);
        assert_eq!(storage.get_last_log_message_time().unwrap(), 40_u64);
    }

    fn validate_message(message: &LogMessage, nanos: Nanos, text: &str) {
        assert_eq!(message.timeNanos, nanos);
        assert_eq!(message.message, text);
    }
}
