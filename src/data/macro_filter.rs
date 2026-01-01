use std::error::Error;

pub struct MacroFilter {
    pub is_bull_market: bool,
    pub rsi_oversold: bool,
}

impl MacroFilter {
    pub async fn get_market_context() -> Result<Self, Box<dyn Error>> {
        // En una versión futura, aquí usaremos reqwest para bajar el CSV de Yahoo
        // Por ahora, devolvemos un estado por defecto para que el bot compile y corra
        
        Ok(Self {
            is_bull_market: true,   // El bot asume mercado alcista por defecto
            rsi_oversold: false,
        })
    }
}