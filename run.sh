docker run -d --name=yahoo-finance-metrics -p 9101:8080 -v $(pwd)/config.json:/etc/yahoo-finance-metrics/config.json yahoo-finance-metrics

