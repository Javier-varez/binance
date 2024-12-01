use crate::utils::LazyF64;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceChange24Hr<'a> {
    pub symbol: &'a str,
    pub price_change: LazyF64<'a>,
    pub price_change_percent: LazyF64<'a>,
    pub last_price: LazyF64<'a>,
    // Initially I tried to also make u64 lazy fields, but it seems to be slower because these
    // values are handled as numbers not strings by serde.
    pub last_qty: LazyF64<'a>,
    pub open: LazyF64<'a>,
    pub high: LazyF64<'a>,
    pub low: LazyF64<'a>,
    pub volume: LazyF64<'a>,
    pub amount: LazyF64<'a>,
    pub bid_price: LazyF64<'a>,
    pub ask_price: LazyF64<'a>,
    pub open_time: u64,
    pub close_time: u64,
    pub first_trade_id: u64,
    pub trade_count: u64,
    pub strike_price: LazyF64<'a>,
    pub exercise_price: LazyF64<'a>,
}

/// Parses the data returned by the `GET /eapi/v1/ticker` endpoint
pub fn parse(data: &str) -> anyhow::Result<Vec<PriceChange24Hr>> {
    Ok(serde_json::from_str(data)?)
}
