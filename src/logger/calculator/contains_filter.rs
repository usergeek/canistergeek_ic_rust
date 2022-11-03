use super::super::data_type::LogMessage;
use super::Filter;

/// Implementation Filter for filter message by contains text matching

pub struct MessageContainsFilter {
    contains_text: String,
    analyze_count: usize,
    analyzed: usize,
}

impl MessageContainsFilter {
    pub fn create(
        analyze_count: usize,
        contains_text: &str,
    ) -> Result<MessageContainsFilter, &str> {
        Ok(MessageContainsFilter {
            analyze_count,
            contains_text: contains_text.to_lowercase(),
            analyzed: 0,
        })
    }
}

impl Filter for MessageContainsFilter {
    fn check_match(&mut self, log_message: &LogMessage) -> bool {
        self.analyzed += 1;
        log_message
            .message
            .to_lowercase()
            .contains(&self.contains_text)
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
        let mut filter = super::MessageContainsFilter::create(3, "Abc").unwrap();

        assert_eq!(filter.is_stop(), false);
        assert_eq!(
            filter.check_match(&LogMessage {
                timeNanos: 0,
                message: String::from("mess aBc sss")
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
