use crate::{Error, ErrorKind};
#[cfg(feature = "serde")]
use serde::{de, de::Error as _, ser, Deserialize, Serialize};
use std::{fmt, ops::Deref, str::FromStr, time::Duration};

/// Timeout durations
#[derive(Copy, Clone, Debug)]
pub struct Timeout(Duration);

impl Deref for Timeout {
    type Target = Duration;

    fn deref(&self) -> &Duration {
        &self.0
    }
}

impl From<Duration> for Timeout {
    fn from(duration: Duration) -> Timeout {
        Timeout(duration)
    }
}

impl From<Timeout> for Duration {
    fn from(timeout: Timeout) -> Duration {
        timeout.0
    }
}

impl FromStr for Timeout {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Timeouts are either 'ms' or 's', and should always end with 's'
        if s.len() < 2 || !s.ends_with('s') {
            Err(err!(ErrorKind::Parse, "invalid units"))?;
        }

        let units = match s.chars().nth(s.len() - 2) {
            Some('m') => "ms",
            Some('0'..='9') => "s",
            _ => Err(err!(ErrorKind::Parse, "invalid units"))?,
        };

        let numeric_part = s.chars().take(s.len() - units.len()).collect::<String>();

        let numeric_value = numeric_part
            .parse::<u64>()
            .map_err(|e| err!(ErrorKind::Parse, e))?;

        let duration = match units {
            "s" => Duration::from_secs(numeric_value),
            "ms" => Duration::from_millis(numeric_value),
            _ => unreachable!(),
        };

        Ok(Timeout(duration))
    }
}

impl fmt::Display for Timeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ms", self.as_millis())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Timeout {
    /// Parse `Timeout` from string ending in `s` or `ms`
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let string = String::deserialize(deserializer)?;
        string
            .parse()
            .map_err(|_| D::Error::custom(format!("invalid timeout value: {:?}", &string)))
    }
}

#[cfg(feature = "serde")]
impl Serialize for Timeout {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::Timeout;
    use crate::error::ErrorKind;

    #[test]
    fn parse_seconds() {
        let timeout = "123s".parse::<Timeout>().unwrap();
        assert_eq!(timeout.as_secs(), 123);
    }

    #[test]
    fn parse_milliseconds() {
        let timeout = "123ms".parse::<Timeout>().unwrap();
        assert_eq!(timeout.as_millis(), 123);
    }

    #[test]
    fn reject_no_units() {
        assert_eq!(
            *"123".parse::<Timeout>().err().unwrap().kind(),
            ErrorKind::Parse
        );
    }
}
