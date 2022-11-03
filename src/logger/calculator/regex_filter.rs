use super::super::data_type::LogMessage;
use super::Filter;
use regex::Regex;

/// Implementation Filter for filter message by message Reqex matching

pub struct MessageRegexFilter {
    regex: Regex,
    analyze_count: usize,
    analyzed: usize,
}

impl MessageRegexFilter {
    pub fn create(analyze_count: usize, regex_text: &str) -> Result<MessageRegexFilter, &str> {
        match Regex::new(regex_text) {
            Ok(regex) => Ok(MessageRegexFilter {
                analyze_count,
                regex,
                analyzed: 0,
            }),
            Err(_) => Err("Can not create regex filter"),
        }
    }
}

impl Filter for MessageRegexFilter {
    fn check_match(&mut self, log_message: &LogMessage) -> bool {
        self.analyzed += 1;
        self.regex.is_match(&log_message.message)
    }

    fn is_stop(&self) -> bool {
        self.analyzed >= self.analyze_count
    }
}

#[cfg(test)]
mod tests {
    use super::Filter;
    use crate::logger::data_type::LogMessage;

    #[test]
    fn test() {
        let mut filter = super::MessageRegexFilter::create(3, "abc").unwrap();

        assert_eq!(filter.is_stop(), false);
        assert_eq!(
            filter.check_match(&LogMessage {
                timeNanos: 0,
                message: String::from("mess abc sss")
            }),
            true
        );
        assert_eq!(filter.is_stop(), false);
        assert_eq!(
            filter.check_match(&LogMessage {
                timeNanos: 0,
                message: String::from("aa abc bb")
            }),
            true
        );
        assert_eq!(filter.is_stop(), false);
        assert_eq!(
            filter.check_match(&LogMessage {
                timeNanos: 0,
                message: String::from("aa ab bb")
            }),
            false
        );
        assert_eq!(filter.is_stop(), true);
    }
}
