use headless_chrome::protocol::cdp::Network::CookieParam;
use headless_chrome::{Browser, LaunchOptionsBuilder};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct StockMetrics {
    pub total_holding_value: f32,
    pub daily_gain_percent_value: f32,
    pub daily_gain_value: f32,
    pub total_gain_value: f32,
    pub total_gain_percent_value: f32,
}

pub fn fetch_stock_data() -> Result<(), Box<dyn Error>> {
    let options = LaunchOptionsBuilder::default()
        .window_size(Some((1280, 1280)))
        .headless(true)
        .sandbox(false)
        .port(Some(9222))
        .build()
        .unwrap();
    let browser = Browser::new(options).expect("Unable to create Browser");
    let tab = browser.new_tab()?;
    let config_content = fs::read_to_string("/etc/yahoo-finance-metrics/config.json")?;
    let config: Value = serde_json::from_str(&config_content)?;

    if let Some(cookies_json) = config.get("cookies").and_then(|v| v.as_array()) {
        let cookies: Vec<CookieParam> = cookies_json
            .iter()
            .map(|cookie| {
                serde_json::from_value(cookie.clone()).expect("Unable to parse cookie from config")
            })
            .collect();

        tab.set_cookies(cookies)?;
    } else {
        return Err("Missing or invalid 'cookies' in config".into());
    }

    tab.navigate_to("https://finance.yahoo.com/portfolio/p_0/view/v1")?;
    tab.wait_until_navigated()?;

    let total_holding = tab
        .wait_for_element(
            "div[data-yaft-module=\"tdv2-applet-fin-portfolio-gainloss\"] > div > span",
        )?
        .get_inner_text()
        .expect("Unable to get total holding value");

    let daily_gain = tab.wait_for_element(
        "div[data-yaft-module=\"tdv2-applet-fin-portfolio-gainloss\"] > div:nth-of-type(3) > span:nth-of-type(2)"
    )?.get_inner_text().expect("Unable to get daily gain value");

    let total_gain = tab.wait_for_element(
        "div[data-yaft-module=\"tdv2-applet-fin-portfolio-gainloss\"] > div:nth-of-type(4) > span:nth-of-type(2)"
    )?.get_inner_text().expect("Unable to get total gain value");

    let total_holding_value = total_holding
        .split_whitespace()
        .nth(0)
        .expect("conversion error 1")
        .trim_matches('$')
        .replace(",", "")
        .parse::<f32>()?;
    let daily_gain_percent_value = daily_gain
        .split_whitespace()
        .nth(1)
        .expect("conversion error 2")
        .trim_matches('(')
        .trim_matches(')')
        .trim_matches('%')
        .parse::<f32>()?;

    let daily_gain_value = daily_gain
        .split_whitespace()
        .nth(0)
        .expect("conversion error 3")
        .parse::<f32>()?;

    let total_gain_value = total_gain
        .split_whitespace()
        .nth(0)
        .expect("conversion error 4")
        .parse::<f32>()?;

    let total_gain_percent_value = total_gain
        .split_whitespace()
        .nth(1)
        .expect("conversion error 5")
        .trim_matches('(')
        .trim_matches(')')
        .trim_matches('%')
        .parse::<f32>()?;

    let metrics = StockMetrics {
        total_holding_value,
        daily_gain_percent_value,
        daily_gain_value,
        total_gain_value,
        total_gain_percent_value,
    };

    println!("Metrics: {:?}", metrics);
    // data is in src/data/metrics.json from where the program is run
    let metrics_json_string = serde_json::to_string(&metrics)?;
    // see if this is a production build or not
    let metrics_path = if cfg!(debug_assertions) {
        "src/data/metrics.json"
    } else {
        "/etc/yahoo-finance-metrics/metrics.json"
    };

    fs::write(metrics_path, metrics_json_string)?;
    Ok(())
}
