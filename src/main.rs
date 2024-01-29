mod utils;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use handlebars::DirectorySourceOptions;
use handlebars::Handlebars;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::process;
use strum::IntoEnumIterator;
use tokio::time::{sleep, Duration};
use utils::get_stock_data::{fetch_stock_data, StockMetrics};
use utils::get_stock_exchange_info::{get_exchange_info, is_exchange_open, Exchange};

#[derive(Debug, Deserialize)]
struct SettingsForm {
    exchange: String,
}

#[get("/")]
async fn hello(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    let config_path = if cfg!(debug_assertions) {
        "src/data/config.json"
    } else {
        "/etc/yahoo-finance-metrics/config.json"
    };

    let config_str = fs::read_to_string(config_path).expect("Unable to read config file");
    let config: serde_json::Value =
        serde_json::from_str(&config_str).expect("Unable to parse config file");

    let data = json!({
        "title": "Yahoo Finance Metrics",
        "message": config["message"].as_str().unwrap_or("Hello, world!"),
    });

    let body = hb.render("index", &data).unwrap();

    HttpResponse::Ok().body(body)
}

#[get("/settings")]
async fn settings(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    let config_path = if cfg!(debug_assertions) {
        "src/data/config.json"
    } else {
        "/etc/yahoo-finance-metrics/config.json"
    };

    let config_str = fs::read_to_string(config_path).expect("Unable to read config file");
    let config: serde_json::Value =
        serde_json::from_str(&config_str).expect("Unable to parse config file");

    let exchange = get_exchange_info(config["exchange"].as_str().unwrap_or("NYSE"));
    let data = json!({
        "exchange": exchange,
        "is_market_open": is_exchange_open(config["exchange"].as_str().unwrap_or("NYSE")),
        "exchanges": Exchange::iter().map(|e| e.to_string()).collect::<Vec<String>>(),
    });

    println!("{:?}", data);

    let body = hb.render("settings", &data).unwrap();

    HttpResponse::Ok().body(body)
}
#[post("/settings")]
async fn update_settings(form: web::Json<SettingsForm>) -> impl Responder {
    let config_path = if cfg!(debug_assertions) {
        "src/data/config.json"
    } else {
        "/etc/yahoo-finance-metrics/config.json"
    };

    let config_str = fs::read_to_string(config_path).expect("Unable to read config file");
    let mut config: serde_json::Value =
        serde_json::from_str(&config_str).expect("Unable to parse config file");
    config["exchange"] = json!(form.exchange);
    let config_str = serde_json::to_string_pretty(&config).expect("Unable to serialize config");

    fs::write(config_path, config_str).expect("Unable to write config file");
    HttpResponse::Ok().json(json!({
       "exchange": get_exchange_info(form.exchange.as_str()),
       "is_market_open": is_exchange_open(form.exchange.as_str()),
       "message": format!("Updated exchange to {}", form.exchange),
    }))
}

#[get("/metrics")]
async fn metrics() -> impl Responder {
    let metrics_path = if cfg!(debug_assertions) {
        "src/data/metrics.json"
    } else {
        "/etc/yahoo-finance-metrics/metrics.json"
    };

    if !std::path::Path::new(metrics_path).exists() {
        let _ = fetch_stock_data();
    }

    let metrics_json = fs::read_to_string(metrics_path).expect("Unable to read metrics file");
    let metrics: StockMetrics = serde_json::from_str(&metrics_json).expect("Unable to parse JSON");
    HttpResponse::Ok().body(format!(
        "yahoo_finance_total_holding_value {}\nyahoo_finance_daily_gain_percent_value {}\nyahoo_finance_daily_gain_value {}\nyahoo_finance_total_gain_value {}\nyahoo_finance_total_gain_percent_value {}",
        metrics.total_holding_value,
        metrics.daily_gain_percent_value,
        metrics.daily_gain_value,
        metrics.total_gain_value,
        metrics.total_gain_percent_value,
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    actix_rt::spawn(async {
        loop {
            let config_path = if cfg!(debug_assertions) {
                "src/data/config.json"
            } else {
                "/etc/yahoo-finance-metrics/config.json"
            };
            let config_str = fs::read_to_string(config_path).expect("Unable to read config file");
            let config: serde_json::Value =
                serde_json::from_str(&config_str).expect("Unable to parse config file");
            let is_open = is_exchange_open(config["exchange"].as_str().unwrap_or("NYSE"));
            if is_open {
                let _ = fetch_stock_data();
            } else {
                println!("Market is closed");
            }
            sleep(Duration::from_secs(15)).await;
        }
    });
    let mut handlebars = Handlebars::new();
    // should be a path to templates directory
    let options: DirectorySourceOptions = DirectorySourceOptions {
        tpl_extension: ".html".to_string(),
        hidden: false,
        temporary: true,
    };
    handlebars
        .register_templates_directory("src/static/templates", options)
        .unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(handlebars.clone()))
            .service(hello)
            .service(metrics)
            .service(settings)
            .service(update_settings)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await;

    // Kill the process when the program is ended
    process::exit(server.is_err() as i32);
}
