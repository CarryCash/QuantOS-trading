pub struct MarketBuffer {
    pub prices: Vec<f64>,
    pub highs: Vec<f64>,
    pub lows: Vec<f64>,
    pub volumes: Vec<f64>,
    pub limit: usize,
}

impl MarketBuffer {
    pub fn new(limit: usize) -> Self {
        Self {
            prices: Vec::with_capacity(limit),
            highs: Vec::with_capacity(limit),
            lows: Vec::with_capacity(limit),
            volumes: Vec::with_capacity(limit),
            limit,
        }
    }

    pub fn add_candle(&mut self, price: f64, volume: f64) {
        if self.prices.len() >= self.limit {
            self.prices.remove(0);
            self.highs.remove(0);
            self.lows.remove(0);
            self.volumes.remove(0);
        }
        
        

        // Como aggTrade nos da el precio actual, en este milisegundo
        // el high y low inicial son el mismo precio.
        self.prices.push(price);
        self.highs.push(price); 
        self.lows.push(price);
        self.volumes.push(volume);
    }

    pub fn get_atrp(&self) -> f64 {
    let count = self.prices.len();
    if count < 2 { return 0.015; } // Valor base de seguridad

    let mut total_tr = 0.0;
    let mut movements = 0;

    for i in 1..count {
        let prev_close = self.prices[i-1];
        let current_close = self.prices[i];
        
        // En micro-velas de 1s, el TR más fiable es el salto entre cierres
        let tr = (current_close - prev_close).abs();
        
        if tr > 0.0 {
            total_tr += tr;
            movements += 1;
        }
    }

    // Si no hay movimiento
    if movements == 0 { return 0.005; }

    let avg_tr = total_tr / movements as f64;
    let current_price = *self.prices.last().unwrap_or(&1.0);

    // Retornamos el porcentaje
    (avg_tr / current_price) * 100.0
}
    pub fn get_features(&self) -> Option<Vec<f64>> {
        if self.prices.len() < self.limit {
            return None;
        }

        let current_price = *self.prices.last()?;
        let prev_price = self.prices[self.prices.len() - 2];

        // 1. Cambio porcentual instantáneo
        let pct_change = (current_price - prev_price) / prev_price;

        // 2. Media Móvil Simple (SMA)
        let sma: f64 = self.prices.iter().sum::<f64>() / self.prices.len() as f64;

        // 3. Desviación del precio
        let price_dev = (current_price - sma) / sma;

        // 4. Efficiency Ratio (ER)
        let net_change = (current_price - self.prices[0]).abs();
        let sum_abs_diffs: f64 = self.prices.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
        let er = if sum_abs_diffs != 0.0 { net_change / sum_abs_diffs } else { 0.0 };

        // 5. Momentum de volumen
        let current_vol = *self.volumes.last()?;
        let avg_vol: f64 = self.volumes.iter().sum::<f64>() / self.volumes.len() as f64;
        let vol_momentum = if avg_vol != 0.0 { current_vol / avg_vol } else { 1.0 };

        // 6. Volatilidad de los retornos
        let log_ret = (current_price / prev_price).ln();

        // 7. Rango de precio en el buffer (Usando nuestros nuevos vectores de High/Low)
        let high = self.highs.iter().fold(f64::MIN, |a, &b| a.max(b));
        let low = self.lows.iter().fold(f64::MAX, |a, &b| a.min(b));
        let range = (high - low) / low;

        // 8. Distancia al High
        let dist_high = (high - current_price) / high;

        Some(vec![pct_change, sma, price_dev, er, vol_momentum, log_ret, range, dist_high])
    }
}