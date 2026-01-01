use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use serde::Deserialize;
use std::time::Duration;

// Estructura para parsear el JSON de Binance
#[derive(Debug, Deserialize)]
pub struct BinanceAggTrade {
    #[serde(rename = "p")]
    pub price: String,
    #[serde(rename = "q")]
    pub quantity: String,
}

// Asegúrate de que esta estructura coincida con lo que espera tu MarketBuffer
#[derive(Clone)]
pub struct PriceMessage {
    pub price: f64,
    pub volume: f64,
}

pub async fn start_market_stream(tx: UnboundedSender<PriceMessage>) {
    let url = "wss://stream.binance.com:9443/ws/btcusdt@aggTrade";

    loop {
        println!("📡 Conectando al WebSocket de Binance (Testnet)...");

        match connect_async(url).await {
            Ok((mut ws_stream, _)) => {
                println!("✅ Conexión establecida.");
                let mut ping_interval = tokio::time::interval(Duration::from_secs(20));

                loop {
                    tokio::select! {
                        // 1. Recibir mensajes de Binance
                        msg = ws_stream.next() => {
                            match msg {
                                Some(Ok(Message::Text(text))) => {
                                    if let Ok(parsed) = serde_json::from_str::<BinanceAggTrade>(&text) {
                                        let price = parsed.price.parse::<f64>().unwrap_or(0.0);
                                        let volume = parsed.quantity.parse::<f64>().unwrap_or(0.0);
                                        
                                        // Enviamos los datos limpios al main.rs
                                        let _ = tx.send(PriceMessage { price, volume });
                                    }
                                }
                                Some(Ok(Message::Ping(payload))) => {
                                    // Respuesta inmediata al Ping de Binance
                                    let _ = ws_stream.send(Message::Pong(payload)).await;
                                }
                                Some(Err(e)) => {
                                    println!("❌ Error en el stream: {:?}", e);
                                    break;
                                }
                                None => break, // Conexión cerrada
                                _ => {}
                            }
                        }
                        // 2. Pilar 14: Ping proactivo para evitar desconexiones por inactividad
                        _ = ping_interval.tick() => {
                            if let Err(e) = ws_stream.send(Message::Ping(vec![])).await {
                                println!("❌ Fallo al enviar Ping proactivo: {:?}", e);
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Error de conexión: {:?}. Reintentando...", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}