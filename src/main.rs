mod utils;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use serde_json;
use std::fs;
use std::process;
use tokio::time::{sleep, Duration};
use utils::get_stock_data::{fetch_stock_data, StockMetrics};

#[get("/")]
async fn hello() -> impl Responder {
    let config_str = fs::read_to_string("/etc/yahoo-finance-metrics/config.json")
        .expect("Unable to read config file");
    let config: serde_json::Value =
        serde_json::from_str(&config_str).expect("Unable to parse config file");

    let msg = format!(
        "Hello from {}!\n",
        config["message"].as_str().unwrap_or("Unknown")
    );

    HttpResponse::Ok().body(msg)
}

#[get("/metrics")]
async fn metrics() -> impl Responder {
    // see if this is a production build or not
    let metrics_path = if cfg!(debug_assertions) {
        "src/data/metrics.json"
    } else {
        "/etc/yahoo-finance-metrics/metrics.json"
    };
    let metrics_json = fs::read_to_string(metrics_path).expect("Unable to read metrics file");
    let metrics: StockMetrics = serde_json::from_str(&metrics_json).expect("Unable to parse JSON");
    HttpResponse::Ok().body(format!(
        "yahoo_finance_total_holding_value {}\nyahoo_finance_daily_gain_percent_value {}\nyahoo_finance_daily_gain_value {}\nyahoo_finance_total_gain_value {}\nyahoo_finance_total_gain_percent_value {}",
        metrics.total_holding_value,
        metrics.daily_gain_percent_value,
        metrics.daily_gain_value,
        metrics.total_gain_value,
        metrics.total_gain_percent_value
    ))
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    actix_rt::spawn(async {
        loop {
            fetch_stock_data().expect("Unable to fetch stock data");
            sleep(Duration::from_secs(15)).await;
        }
    });

    let server =
        HttpServer::new(|| App::new().service(hello).service(metrics)).bind(("0.0.0.0", 8080))?;

    let result = server.run().await;

    // Kill the process when the program is ended
    process::exit(result.is_err() as i32);
}
