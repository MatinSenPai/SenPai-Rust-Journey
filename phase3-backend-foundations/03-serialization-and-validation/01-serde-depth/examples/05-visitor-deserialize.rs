//! A hand-written `Deserialize` with a `Visitor`: `"24m"` or `24` becomes `Minutes`.
//! Run: `cargo run -p p3-03-01-serde-depth --example 05-visitor-deserialize`

use serde::de::{self, Deserialize, Deserializer, Visitor};
use std::fmt;

#[derive(Debug, PartialEq)]
struct Minutes(u32);

struct MinutesVisitor;

impl<'de> Visitor<'de> for MinutesVisitor {
    type Value = Minutes;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a number of minutes like 24, or a string like \"24m\"")
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Minutes, E> {
        u32::try_from(v).map(Minutes).map_err(E::custom)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Minutes, E> {
        let digits = v
            .strip_suffix('m')
            .ok_or_else(|| E::custom("missing `m`"))?;
        digits.parse().map(Minutes).map_err(E::custom)
    }
}

impl<'de> Deserialize<'de> for Minutes {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Minutes, D::Error> {
        d.deserialize_any(MinutesVisitor)
    }
}

fn main() {
    for input in ["24", "\"24m\"", "\"24\"", "true", "-3"] {
        println!("{input:>6} -> {:?}", serde_json::from_str::<Minutes>(input));
    }
}
