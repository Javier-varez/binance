fn get_ticker_price_change(base_endpoint: &str) -> anyhow::Result<String> {
    let ticker_endpoint = format!("{base_endpoint}/eapi/v1/ticker");
    let request = reqwest::blocking::get(ticker_endpoint)?;
    for (k, v) in request.headers().iter() {
        println!("{k:?}: {v:?}");
    }
    let request = request.error_for_status()?;

    Ok(request.text()?)
}

fn main() -> anyhow::Result<()> {
    let endpoint_result = get_ticker_price_change("https://eapi.binance.com")?;

    let price_changes = binance::custom::parse(&endpoint_result)?;
    for i in &price_changes {
        println!("{:#?}", i);
    }

    // An example showing how to use the lazy API.
    let document = binance::custom_lazy::Document::new(&endpoint_result);
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
