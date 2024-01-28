use headless_chrome::protocol::cdp::Network::CookieParam;
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fs;

use std::sync::Arc;

fn fetch_text(tab: &Arc<Tab>, selector: &str) -> Result<Option<String>, Box<dyn Error>> {
    match tab.wait_for_element(selector) {
        Ok(element) => match element.get_inner_text() {
            Ok(text) => Ok(Some(text)),
            Err(e) => {
                eprintln!("Error getting inner text: {}", e);
                Ok(None)
            }
        },
        Err(_) => {
            eprintln!("Element not found for selector: {}", selector);
            Ok(None)
        }
    }
}

fn parse_value(
    text: Option<String>,
    index: usize,
    trim_chars: &[char],
) -> Result<Option<f32>, Box<dyn Error>> {
    match text {
        Some(t) => {
            let value = t
                .split_whitespace()
                .nth(index)
                .ok_or(format!("Failed to parse value at index: {}", index))?
                .trim_matches(trim_chars)
                .replace(",", "")
                .parse::<f32>()?;
            Ok(Some(value))
        }
        None => Ok(None), // No text to parse, so return None
    }
}

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

    let total_holding = fetch_text(
        &tab,
        "div[data-yaft-module=\"tdv2-applet-fin-portfolio-gainloss\"] > div > span",
    );
    let daily_gain = fetch_text(&tab, "div[data-yaft-module=\"tdv2-applet-fin-portfolio-gainloss\"] > div:nth-of-type(3) > span:nth-of-type(2)");
    let total_gain = fetch_text(&tab, "div[data-yaft-module=\"tdv2-applet-fin-portfolio-gainloss\"] > div:nth-of-type(4) > span:nth-of-type(2)");

    let total_holding_value = parse_value(total_holding?, 0, &['$', ','])?;
    let total_gain_value = parse_value(total_gain?, 0, &['$', ','])?;
    let daily_gain_value = parse_value(daily_gain?, 0, &['$', ','])?;
    let total_gain_percent_value = parse_value(total_gain?, 1, &['$', ',', '%'])?;
    let daily_gain_percent_value = parse_value(daily_gain?, 1, &['$', ',', '%'])?;

    if total_holding_value.is_none()
        && daily_gain_percent_value.is_none()
        && daily_gain_value.is_none()
        && total_gain_value.is_none()
        && total_gain_percent_value.is_none()
    {
        println!("All metrics are None. Skipping file write.");
    } else {
        let metrics = StockMetrics {
            total_holding_value: total_holding_value.unwrap_or(0.0),
            daily_gain_percent_value: daily_gain_percent_value.unwrap_or(0.0),
            daily_gain_value: daily_gain_value.unwrap_or(0.0),
            total_gain_value: total_gain_value.unwrap_or(0.0),
            total_gain_percent_value: total_gain_percent_value.unwrap_or(0.0),
        };
        println!("Metrics: {:?}", metrics);
        let metrics_json_string = serde_json::to_string(&metrics)?;
        let metrics_path = if cfg!(debug_assertions) {
            "src/data/metrics.json"
        } else {
            "/etc/yahoo-finance-metrics/metrics.json"
        };

        fs::write(metrics_path, metrics_json_string)?;
    }
    Ok(())
}
