#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceChange24Hr<'a> {
    pub symbol: &'a str,
    pub price_change: &'a str,
    pub price_change_percent: &'a str,
    pub last_price: &'a str,
    pub last_qty: &'a str,
    pub open: &'a str,
    pub high: &'a str,
    pub low: &'a str,
    pub volume: &'a str,
    pub amount: &'a str,
    pub bid_price: &'a str,
    pub ask_price: &'a str,
    pub open_time: usize,
    pub close_time: usize,
    pub first_trade_id: usize,
    pub trade_count: usize,
    pub strike_price: &'a str,
    pub exercise_price: &'a str,
}

/// Parses the data returned by the `GET /eapi/v1/ticker` endpoint
pub fn parse(data: &str) -> anyhow::Result<Vec<PriceChange24Hr>> {
    Ok(serde_json::from_str(data)?)
}
