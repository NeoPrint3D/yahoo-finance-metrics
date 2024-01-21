# Yahoo Finance Metrics for portfolios

![Grafana Dashboard](assets/thumb.png)

This is a simple python script that scrapes the Yahoo Finance website for the current price current value of your portfolio and tells you some key metrics about it.

## Info

- yahoo_finance_total_holding_value ($)
- yahoo_finance_daily_gain_percent_value (%)
- yahoo_finance_daily_gain_value ($)
- yahoo_finance_total_gain_value ($)
- yahoo_finance_total_gain_percent_value (%)

## Prerequisites

In order to use this you must get the cookies for your Yahoo Finance account. You can do this by logging into your account and then using the developer tools in your browser to get the cookies.

## Running

### Docker

```bash
docker pull np3d/yahoo-finance-metrics
# create a new directory to store the config file
mkdir /yourfolder
cd /yourfolder
# copy the example config
## Development
wget https://github.com/NeoPrint3D/yahoo-finance-metrics/blob/master/config_example.json?raw=true -O config.json

docker run -d --name=np3d/yahoo-finance-metrics -p 9101:8080 -v $(pwd)/config.json:/etc/yahoo-finance-metrics/config.json yahoo-finance-metrics
```

1. Clone this repo
2. Install the requirements
3. Run the script

```bash
git clone
cd yahoo-finance-metrics
```

### Docker Usage

```bash
docker build -t yahoo-finance-metrics .
sh run.sh
```

### Rust Usage

- Must have rustup installed with cargo and rustc
- Must place the config.jsonn file into /etc/yahoo-finance-metrics/config.json

```bash
cargo build --release
# Run the binary
./target/release/yahoo-finance-metrics
```

## TODO

- [ ] Add more metrics
- [x] Only scrapes when the market is open
- [ ] Add more support for different OS's
- [ ] Add more support for more than one portfolios (currently only supports one which is the first one in the list)
- [x] public docker container
