use serde_json::Value;

pub async fn is_stock_market_open() -> Result<bool, Box<dyn std::error::Error>> {
    let res: Value = reqwest::get("https://financialmodelingprep.com/api/v3/is-the-market-open?apikey=GAbjhrSDKkoDe2qdFNOgiVCSn0XIO2IK")
        .await
        .expect("Unable to fetch market data")
        .json()
        .await
        .expect("Unable to parse market data");
    let is_open = res["isTheStockMarketOpen"]
        .as_bool()
        .expect("Unable to parse JSON");
    Ok(is_open)
}
