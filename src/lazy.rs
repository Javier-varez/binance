#[derive(serde::Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct LazyF64<'a>(&'a str);

#[derive(serde::Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct LazyU64<'a>(&'a str);

impl<'a> TryInto<u64> for LazyU64<'a> {
    type Error = ();
    fn try_into(self) -> Result<u64, Self::Error> {
        self.0.parse().map_err(|_| ())
    }
}

impl<'a> TryInto<f64> for LazyF64<'a> {
    type Error = ();
    fn try_into(self) -> Result<f64, Self::Error> {
        self.0.parse().map_err(|_| ())
    }
}

impl<'a> std::fmt::Debug for LazyF64<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result: Result<f64, _> = (*self).try_into();
        write!(f, "{:?}", result)
    }
}

impl<'a> std::fmt::Debug for LazyU64<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result: Result<u64, _> = (*self).try_into();
        write!(f, "{:?}", result)
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceChange24Hr<'a> {
    pub symbol: &'a str,
    pub price_change: LazyF64<'a>,
    pub price_change_percent: LazyF64<'a>,
    pub last_price: LazyF64<'a>,
    pub last_qty: LazyU64<'a>,
    pub open: LazyF64<'a>,
    pub high: LazyF64<'a>,
    pub low: LazyF64<'a>,
    pub volume: LazyU64<'a>,
    pub amount: LazyU64<'a>,
    pub bid_price: LazyF64<'a>,
    pub ask_price: LazyF64<'a>,
    pub open_time: LazyU64<'a>,
    pub close_time: LazyU64<'a>,
    pub first_trade_id: LazyU64<'a>,
    pub trade_count: LazyU64<'a>,
    pub strike_price: LazyF64<'a>,
    pub exercise_price: LazyF64<'a>,
}

/// This slightly improved parsing function returns an array that borrows all
/// the data.
/// It also lazily parses most fields, except for those numeric fields
/// inside json strings.
pub fn parse(data: &str) -> anyhow::Result<Vec<PriceChange24Hr>> {
    Ok(serde_json::from_str(data)?)
}
