
pub struct PositionManager {
    pub balance_usd: f64,
    pub risk_percentage: f64, // Ej: 0.02 para 2%
    pub highest_price: f64,
}

impl PositionManager {
    pub fn new(balance: f64, risk: f64) -> Self {
        Self { balance_usd: balance, risk_percentage: risk, highest_price: 0.0, }
    }

    /// Calcula la cantidad de BTC a comprar basada en la distancia al Stop Loss
    pub fn calculate_order_size(&self, entry_price: f64, stop_loss: f64) -> f64 {
        let amount_to_risk = self.balance_usd * self.risk_percentage;
        let risk_per_unit = (entry_price - stop_loss).abs();

        if risk_per_unit == 0.0 { return 0.0; }

        let btc_quantity = amount_to_risk / risk_per_unit;
        
        // Limitar para no exceder el balance total (leverage 1x por ahora)
        let max_possible = self.balance_usd / entry_price;
        if btc_quantity > max_possible { max_possible } else { btc_quantity }
    }

    // En src/trading/position_manager.rs
    pub fn update_highest_price(&mut self, current_price: f64) { // <-- Debe tener &mut
        if current_price > self.highest_price {
            self.highest_price = current_price;
        }
    }

    pub fn calculate_trailing_stop(&self, trail_percent: f64) -> f64 {
        self.highest_price * (1.0 - trail_percent)
    }

    pub fn reset_position(&mut self) {
        self.highest_price = 0.0;
    }
}