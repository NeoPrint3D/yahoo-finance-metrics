use chrono::{Datelike, Local, Timelike};
use chrono_tz::Tz;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use strum_macros::{Display, EnumIter};

fn serialize_tz<S>(tz: &Tz, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&tz.name())
}

// Custom deserialization for the Tz field
fn deserialize_tz<'de, D>(deserializer: D) -> Result<Tz, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeDetails {
    pub name: &'static str,
    pub open_hour: f32,
    pub close_hour: f32,
    #[serde(serialize_with = "serialize_tz", deserialize_with = "deserialize_tz")]
    pub time_zone: Tz,
}

// Define known exchanges
#[derive(EnumIter, Display)]
pub enum Exchange {
    NYSE,
    NASDAQ,
    TSX,
    LSE,
    TSE,
}

impl Exchange {
    fn details(&self) -> ExchangeDetails {
        match self {
            Exchange::NYSE => ExchangeDetails {
                name: "NYSE",
                open_hour: 9.5,
                close_hour: 16.0,
                time_zone: chrono_tz::America::New_York,
            },
            Exchange::NASDAQ => ExchangeDetails {
                name: "NASDAQ",
                open_hour: 9.5,
                close_hour: 16.0,
                time_zone: chrono_tz::America::New_York,
            },
            Exchange::TSX => ExchangeDetails {
                name: "TSX",
                open_hour: 9.5,
                close_hour: 16.0,
                time_zone: chrono_tz::America::Toronto,
            },
            Exchange::LSE => ExchangeDetails {
                name: "LSE",
                open_hour: 8.0,
                close_hour: 16.0,
                time_zone: chrono_tz::Europe::London,
            },
            Exchange::TSE => ExchangeDetails {
                name: "TSE",
                open_hour: 9.5,
                close_hour: 15.0,
                time_zone: chrono_tz::Asia::Tokyo,
            },
        }
    }
}

pub fn is_exchange_open(exchange_name: &str) -> bool {
    let exchange = match exchange_name {
        "NYSE" => Exchange::NYSE,
        "NASDAQ" => Exchange::NASDAQ,
        "TSX" => Exchange::TSX,
        "LSE" => Exchange::LSE,
        "TSE" => Exchange::TSE,
        _ => {
            println!("Exchange not recognized.");
            return false; // Return false if the exchange is not recognized
        }
    };

    let exchange_details = exchange.details();
    let local_time = Local::now().with_timezone(&exchange_details.time_zone);

    // Convert current hour and minute to a float (e.g., 9.5 for 9:30)
    let current_time_in_hours = local_time.hour() as f32 + local_time.minute() as f32 / 60.0;

    let is_within_hours = current_time_in_hours >= exchange_details.open_hour
        && current_time_in_hours < exchange_details.close_hour;

    let weekday = local_time.weekday();

    // Placeholder for holiday check
    let is_holiday = false; // Implement holiday check based on your data source

    matches!(
        weekday,
        chrono::Weekday::Mon
            | chrono::Weekday::Tue
            | chrono::Weekday::Wed
            | chrono::Weekday::Thu
            | chrono::Weekday::Fri
    ) && is_within_hours
        && !is_holiday
}

pub fn get_exchange_info(exchange_name: &str) -> ExchangeDetails {
    let exchange = match exchange_name {
        "NYSE" => Exchange::NYSE,
        "NASDAQ" => Exchange::NASDAQ,
        "TSX" => Exchange::TSX,
        "LSE" => Exchange::LSE,
        "TSE" => Exchange::TSE,
        _ => {
            println!("Exchange not recognized.");
            Err("Exchange not recognized.").unwrap()
        }
    };
    exchange.details()
}
