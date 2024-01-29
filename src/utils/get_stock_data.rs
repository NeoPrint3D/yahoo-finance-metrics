use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, sync::Arc};

#[derive(Debug, Serialize, Deserialize)]
pub struct StockMetrics {
    pub total_holding_value: f32,
    pub daily_gain_percent_value: f32,
    pub daily_gain_value: f32,
    pub total_gain_value: f32,
    pub total_gain_percent_value: f32,
}

fn fetch_text(tab: &Arc<Tab>, selector: &str) -> Option<String> {
    tab.wait_for_element(selector).ok()?.get_inner_text().ok()
}

fn parse_value(text: Option<String>, index: usize, trim_chars: &[char]) -> Option<f32> {
    text?
        .split_whitespace()
        .nth(index)?
        .trim_matches(trim_chars)
        .replace(",", "")
        .parse::<f32>()
        .ok()
}

pub fn fetch_stock_data() {
    let options = LaunchOptionsBuilder::default()
        .window_size(Some((1280, 1280)))
        .headless(true)
        .sandbox(false)
        .port(Some(9222))
        .build()
        .expect("Failed to build launch options");

    let browser = match Browser::new(options) {
        Ok(browser) => browser,
        Err(e) => {
            eprintln!("Error creating browser: {}", e);
            return;
        }
    };

    let tab = match browser.new_tab() {
        Ok(tab) => tab,
        Err(e) => {
            eprintln!("Error creating new tab: {}", e);
            return;
        }
    };

    let config_path = if cfg!(debug_assertions) {
        "src/data/config.json"
    } else {
        "/etc/yahoo-finance-metrics/config.json"
    };
    let config_content = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading config file: {}", e);
            return;
        }
    };

    let config: Value = match serde_json::from_str(&config_content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error parsing config JSON: {}", e);
            return;
        }
    };

    let cookies: Vec<_> = match config.get("cookies").and_then(|v| v.as_array()) {
        Some(cookies_json) => cookies_json
            .iter()
            .filter_map(|cookie| serde_json::from_value(cookie.clone()).ok())
            .collect(),
        None => {
            eprintln!("Missing or invalid 'cookies' in config");
            return;
        }
    };

    if let Err(e) = tab.set_cookies(cookies) {
        eprintln!("Error setting cookies: {}", e);
        return;
    }

    if let Err(e) = tab.navigate_to("https://finance.yahoo.com/portfolio/p_0/view/v1") {
        eprintln!("Error navigating to URL: {}", e);
        return;
    }

    tab.wait_until_navigated().expect("Failed to navigate");

    let selectors = vec![
        "div[data-yaft-module=\"tdv2-applet-fin-portfolio-gainloss\"] > div > span",
        "div[data-yaft-module=\"tdv2-applet-fin-portfolio-gainloss\"] > div:nth-of-type(3) > span:nth-of-type(2)",
        "div[data-yaft-module=\"tdv2-applet-fin-portfolio-gainloss\"] > div:nth-of-type(4) > span:nth-of-type(2)",
    ];

    let values: Vec<_> = selectors
        .iter()
        .map(|&selector| fetch_text(&tab, selector))
        .collect();

    if values.iter().any(|v| v.is_none()) {
        eprintln!("Failed to fetch some values");
        return;
    }

    // format -13.68 (-0.33%)
    let metrics = StockMetrics {
        total_holding_value: parse_value(values[0].clone(), 0, &['$', ',']).unwrap_or(0.0),
        daily_gain_value: parse_value(values[1].clone(), 0, &['$', ',']).unwrap_or(0.0),
        daily_gain_percent_value: parse_value(values[1].clone(), 1, &['(', '%', ')'])
            .unwrap_or(0.0),
        total_gain_value: parse_value(values[2].clone(), 0, &['$', ',']).unwrap_or(0.0),
        total_gain_percent_value: parse_value(values[2].clone(), 1, &['(', '%', ')'])
            .unwrap_or(0.0),
    };

    let metrics_path = if cfg!(debug_assertions) {
        "src/data/metrics.json"
    } else {
        "/etc/yahoo-finance-metrics/metrics.json"
    };
    let metrics_json = serde_json::to_string_pretty(&metrics).expect("Unable to serialize metrics");
    fs::write(metrics_path, metrics_json).expect("Unable to write metrics file");
    println!("Metrics: {:?}", metrics);
}
