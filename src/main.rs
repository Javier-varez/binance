fn get_ticker_price_change(base_endpoint: &str) -> anyhow::Result<String> {
    let ticker_endpoint = format!("{base_endpoint}/eapi/v1/ticker");
    let request = reqwest::blocking::get(ticker_endpoint)?;
    for (k, v) in request.headers().iter() {
        println!("{k:?}: {v:?}");
    }
    let request = request.error_for_status()?;

    return Ok(request.text()?);
}

fn main() -> anyhow::Result<()> {
    // let endpoint_result = std::fs::read("request.txt")?;
    // let endpoint_result = std::str::from_utf8(&endpoint_result[..])?;

    let endpoint_result = get_ticker_price_change("https://eapi.binance.com")?;

    let price_changes = binance::naive::parse(&endpoint_result)?;
    for i in &price_changes {
        println!("{:#?}", i);
    }
    println!("count: {}", price_changes.len());

    // An example showing how to use the lazy API.
    let document = binance::very_lazy::Document::new(&endpoint_result);
    let symbol = document
        .as_array()?
        .get_index(0)?
        .as_object()?
        .get_key("symbol")?
        .as_string()?
        .get_value()?;
    println!("symbol: {symbol}");

    Ok(())
}
