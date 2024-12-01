use crate::utils::LazyF64;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceChange24Hr<'a> {
    pub symbol: &'a str,
    pub price_change: LazyF64<'a>,
    pub price_change_percent: LazyF64<'a>,
    pub last_price: LazyF64<'a>,
    // Initially I tried to also make u64 lazy fields, but it seems to be slower because these
    // values are handled as numbers not strings by serde.
    pub last_qty: u64,
    pub open: LazyF64<'a>,
    pub high: LazyF64<'a>,
    pub low: LazyF64<'a>,
    pub volume: u64,
    pub amount: u64,
    pub bid_price: LazyF64<'a>,
    pub ask_price: LazyF64<'a>,
    pub open_time: u64,
    pub close_time: u64,
    pub first_trade_id: u64,
    pub trade_count: u64,
    pub strike_price: LazyF64<'a>,
    pub exercise_price: LazyF64<'a>,
}

/// An attempt to make parsing faster by not having to parse floating point numbers right away.
/// Complexity of this algorithm is linear with the number of entries in the JSON array.
/// However, it should benefit from not having to transform each string to float numbers.
pub fn parse(data: &str) -> anyhow::Result<Vec<PriceChange24Hr>> {
    Ok(serde_json::from_str(data)?)
}
