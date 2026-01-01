use binance::account::*;
use binance::api::*;
use binance::config::Config;
use tokio::task;
use serde_json;
use reqwest;

pub struct Executor {
    api_key: String,
    secret_key: String,
}

impl Executor {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self { api_key, secret_key }
    }

    pub async fn execute_buy(&self, symbol: &str, qty: f64) -> bool {
        let key = self.api_key.clone();
        let secret = self.secret_key.clone();
        let symbol_str = symbol.to_string();
        
        // En Spot BTCUSDT, usamos 5 decimales para mayor precisión
        let formatted_qty = (qty * 100000.0).round() / 100000.0;

        let result = task::spawn_blocking(move || {
            let mut config = Config::default();
            // URL DE SPOT TESTNET (La que sí funciona siempre)
            config.rest_api_endpoint = "https://testnet.binance.vision".to_string();
            
            let account: Account = Binance::new_with_config(Some(key), Some(secret), &config);
            account.market_buy(symbol_str, formatted_qty)
        }).await.unwrap();

        match result {
            Ok(_) => { println!("\n🚀 COMPRA SPOT EXITOSA"); true }
            Err(e) => { println!("\n❌ ERROR SPOT: {:?}", e); false }
        }
    }

    pub async fn execute_sell(&self, symbol: &str, qty: f64) -> bool {
        let key = self.api_key.clone();
        let secret = self.secret_key.clone();
        let symbol_str = symbol.to_string();
        let formatted_qty = (qty * 100000.0).round() / 100000.0;

        let result = task::spawn_blocking(move || {
            let mut config = Config::default();
            config.rest_api_endpoint = "https://testnet.binance.vision".to_string();
            
            let account: Account = Binance::new_with_config(Some(key), Some(secret), &config);
            account.market_sell(symbol_str, formatted_qty)
        }).await.unwrap();

        match result {
            Ok(_) => { println!("\n💰 VENTA SPOT EXITOSA"); true }
            Err(e) => { println!("\n❌ ERROR VENTA SPOT: {:?}", e); false }
        }
    }

    pub async fn get_latest_price(&self, symbol: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", symbol);
        let client = reqwest::Client::new();
        let resp = client.get(url).send().await?.json::<serde_json::Value>().await?;
        
        let price_str = resp["price"].as_str().ok_or("No price in JSON")?;
        let price: f64 = price_str.parse()?;
        Ok(price)
    }
}