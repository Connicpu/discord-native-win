use discord;

use std::fmt;
use std::str;

use chrono::{DateTime, NaiveDateTime, Utc};
use itoa;
use serde::{de, ser};

const TIMESTAMP_SHIFT: u64 = 22;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Snowflake(pub u64);

impl Snowflake {
    pub fn timestamp(&self) -> DateTime<Utc> {
        let time = (self.0 >> TIMESTAMP_SHIFT) as i64;
        let time = time + discord::EPOCH;
        let time = NaiveDateTime::from_timestamp(time / 1000, (time % 1000) as u32 * 1_000_000);
        DateTime::from_utc(time, Utc)
    }
}

impl ser::Serialize for Snowflake {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut buffer = [0; 20];
        let len = itoa::write(&mut buffer[..], self.0).unwrap();
        let text = str::from_utf8(&buffer[..len]).unwrap();

        serializer.serialize_str(text)
    }
}

impl<'de> de::Deserialize<'de> for Snowflake {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Snowflake;
            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("a string containing a serialized 64-bit snowflake integer")
            }

            fn visit_str<E>(self, value: &str) -> Result<Snowflake, E>
            where
                E: de::Error,
            {
                value
                    .parse()
                    .map(Snowflake)
                    .map_err(|_| E::invalid_value(de::Unexpected::Str(value), &Visitor))
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}
