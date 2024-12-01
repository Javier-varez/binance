use crate::utils::LazyF64;

#[derive(Debug, sonic_rs::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceChange24Hr<'a> {
    pub symbol: &'a str,
    pub price_change: LazyF64<'a>,
    pub price_change_percent: LazyF64<'a>,
    pub last_price: LazyF64<'a>,
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
    Ok(sonic_rs::from_str(data)?)
}
