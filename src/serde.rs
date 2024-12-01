#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceChange24Hr {
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub price_change: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub price_change_percent: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub last_price: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub last_qty: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub open: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub high: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub low: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub volume: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub amount: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub bid_price: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub ask_price: f64,
    pub open_time: u64,
    pub close_time: u64,
    pub first_trade_id: u64,
    pub trade_count: u64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub strike_price: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub exercise_price: f64,
}

fn deserialize_f64_from_str<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("Expected a &str to deserialize an f64")
        }

        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let v: f64 = v.parse().map_err(serde::de::Error::custom)?;
            Ok(v)
        }
    }
    deserializer.deserialize_str(Visitor)
}

/// This naive parsing function returns an array that owns all the data.
/// It eagerly parses all fields.
pub fn parse(data: &str) -> anyhow::Result<Vec<PriceChange24Hr>> {
    Ok(serde_json::from_str(data)?)
}
