mod constants;
mod brain;
mod data;
mod trading;

use brain::model_loader::QuantosBrain;
use data::data_buffer::MarketBuffer;
use data::binance_client::PriceMessage;
use trading::position_manager::PositionManager;
use trading::executor::Executor;
use tokio::sync::{mpsc, watch};
use std::sync::Arc;
use dotenv::dotenv;
use std::{env, fs, time::Duration};
use std::io::{self, Write};
use std::fs::OpenOptions;
use std::time::Instant;
use chrono;

// ... (Tus imports se mantienen igual)

#[tokio::main]
async fn main() {
    dotenv().ok();
    let log_path = "logs/historial_binance.txt";
    let _ = fs::create_dir_all("logs");

    println!("--- 🟢 QuantOS Core Engine v1.6 (ASYNCHRONOUS ARCHITECTURE) ---");

    // 1. Inicialización de Componentes
    let api_key = env::var("BINANCE_API_KEY").expect("API_KEY error").trim().to_string();
    let secret_key = env::var("BINANCE_SECRET_KEY").expect("SECRET_KEY error").trim().to_string();
    let executor = Arc::new(Executor::new(api_key, secret_key));
    let brain = Arc::new(QuantosBrain::new("models/quantos_brain_v1.pkl").expect("Error IA"));

    // 2. Canales
    let (price_tx, mut price_rx) = mpsc::unbounded_channel::<PriceMessage>();
    let (ui_tx, _ui_rx) = watch::channel(0.0); 
    let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

    // 3. Sensor y Monitor (Igual que antes)
    let tx_ws = price_tx.clone();
    tokio::spawn(async move { data::binance_client::start_market_stream(tx_ws).await; });
    let stop_tx_clone = stop_tx.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(true) = crossterm::event::poll(Duration::from_millis(100)) {
                if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
                    if key.code == crossterm::event::KeyCode::Char('q') { let _ = stop_tx_clone.send(()).await; break; }
                }
            }
        }
    });

    // 5. VARIABLES DE ESTADO (Persistentes)
    let mut buffer = MarketBuffer::new(14);
    let mut risk_manager = PositionManager::new(1000.0, 0.01); 
    let mut is_position_open = false;
    let mut current_qty = 0.0;
    let mut entry_price = 0.0;
    let mut last_tick_time = Instant::now();
    let mut last_candle_time = Instant::now();
    let candle_interval = Duration::from_secs(1);

    // Variables de visualización
    let mut current_prob = 0.5;
    let mut current_conf = 0.0;

    println!("📡 Patrullando mercado con No-Trade Intelligence activo. Presiona 'Q' para salir.");

    loop {
        tokio::select! {
            _ = stop_rx.recv() => {
                if is_position_open { executor.execute_sell("BTCUSDT", current_qty).await; }
                break;
            }

            Some(msg) = price_rx.recv() => {
                last_tick_time = Instant::now();
                let _ = ui_tx.send(msg.price);

                // --- RESAMPLER: Cada 1 segundo actualizamos cerebro y ATR ---
                if last_candle_time.elapsed() >= candle_interval {
                    buffer.add_candle(msg.price, msg.volume);
                    last_candle_time = Instant::now();

                    if let Some(features) = buffer.get_features() {
                        if let Ok(prob) = brain.predict_noise(features) {
                            current_prob = prob;
                            
                            // Cálculo de Confianza y ATR
                            let atrp = buffer.get_atrp();
                            current_conf = calculate_confidence_score(prob, msg.volume, true, true);
                            
                            let max_spread_allowed = atrp * 0.15;
                            let current_spread_pct = 0.02; // Simulación

                            // LÓGICA DE ENTRADA
                            if current_conf >= 0.75 && !is_position_open {
                                if current_spread_pct <= max_spread_allowed {
                                    let risk_multiplier = if current_conf >= 0.95 { 2.5 } else if current_conf >= 0.90 { 1.8 } else { 1.0 };
                                    let base_size = risk_manager.calculate_order_size(msg.price, msg.price * 0.99);
                                    let dynamic_size = base_size * risk_multiplier;

                                    if dynamic_size > 0.0 && executor.execute_buy("BTCUSDT", dynamic_size).await {
                                        current_qty = dynamic_size;
                                        entry_price = msg.price;
                                        is_position_open = true;
                                        risk_manager.reset_position();
                                        println!("\n🎯 ENTRADA | Conf: {:.2}% | ATR%: {:.3}%", current_conf * 100.0, atrp);
                                    }
                                }
                            }
                        }
                    }
                }

                // LÓGICA DE SALIDA (Se evalúa en cada tick para rapidez)
                if is_position_open {
                    let pnl = (msg.price - entry_price) / entry_price * 100.0;
                    risk_manager.update_highest_price(msg.price);
                    let trail_stop = risk_manager.calculate_trailing_stop(0.005);

                    if msg.price < trail_stop || pnl < -0.8 || current_prob > 0.75 {
                        let motivo = if pnl < -0.8 { "STOP LOSS" } else if current_prob > 0.75 { "NOISE" } else { "TRAIL" };
                        if executor.execute_sell("BTCUSDT", current_qty).await {
                            log_trade(log_path, motivo, entry_price, msg.price, pnl).await;
                            is_position_open = false;
                            println!("\n💰 SALIDA [{}] | PnL: {:.2}%", motivo, pnl);
                        }
                    }
                }

                // UI actualizada en cada tick con los últimos valores del segundo
                refresh_ui(
                    is_position_open, 
                    msg.price, 
                    entry_price, 
                    current_prob, 
                    &risk_manager, 
                    current_conf, 
                    buffer.get_atrp(), 
                    buffer.prices.len(), 
                    buffer.limit
                );
            }

            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                if last_tick_time.elapsed().as_secs() >= 5 && is_position_open {
                    let _ = executor.get_latest_price("BTCUSDT").await;
                }
            }
        }
    }
}

// ... (Tus funciones auxiliares se mantienen igual)

// --- FUNCIONES AUXILIARES ---

fn calculate_confidence_score(prob_ia: f64, volume: f64, is_bull: bool, rsi_oversold: bool) -> f64 {
    let mut score = 0.0;
    if prob_ia < 0.10 { score += 0.55; }
    else if prob_ia < 0.25 { score += 0.45; }
    else if prob_ia < 0.35 { score += 0.30; }

    if volume > 2.0 { score += 0.15; }
    else if volume > 1.0 { score += 0.05; }
    
    if is_bull { score += 0.20; }
    if rsi_oversold { score += 0.10; }
    score
}

async fn log_trade(path: &str, motivo: &str, entry: f64, exit: f64, pnl: f64) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "[{}] {} | In: ${:.2} | Out: ${:.2} | PnL: {:.2}%", 
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), motivo, entry, exit, pnl);
    }
}

fn refresh_ui(is_open: bool, price: f64, entry: f64, prob: f64, _rm: &PositionManager, conf: f64, atrp: f64, current_candles: usize, limit: usize) {

    let progreso = if limit > 0 { (current_candles as f64 / limit as f64 * 10.0) as usize } else { 0 };
    let mut bar = String::from("[");
    for i in 0..10 {
        if i < progreso { bar.push('#'); } else { bar.push('-'); }
    }
    bar.push(']');

    if is_open {
        let pnl = (price - entry) / entry * 100.0;
        // Agregamos {} para la barra al final
        print!("\r🛡️ POSICIÓN | PnL: {:.2}% | IA: {:.4} | ATR%: {:.3}% {} ", 
            pnl, prob, atrp, bar);
    } else {
        // Indicamos si el bot está "CALENTANDO" o "LISTO"
        let status = if current_candles < limit { "CALENTANDO" } else { "LISTO" };
        
        // Corregido: añadidos {} para 'status' y 'bar'
        print!("\r🔍 {} {} | BTC: ${:.2} | IA: {:.4} | Conf: {:.1}% | ATR%: {:.3}% ", 
            status, bar, price, prob, conf * 100.0, atrp);
    }
    let _ = io::stdout().flush();
}