// --- CONSTANTES DEL SISTEMA QuantOS (Pilares 1, 4 y 12) ---

/// Umbral de probabilidad para el No-Trade Intelligence (Pilar 1)
/// Si la IA detecta una probabilidad de ruido superior a esta, el bot se bloquea.
pub const NO_TRADE_THRESHOLD: f64 = 0.70;

/// Umbral para el Risk Engine No Lineal (Pilar 4)
/// Confianza extrema: si el ruido es menor a este valor, operamos al 100%
pub const HIGH_CONFIDENCE_THRESHOLD: f64 = 0.25;

/// Factor de reducción de posición en zona de incertidumbre
pub const RISK_REDUCTION_FACTOR: f64 = 0.10;

/// Comisión estimada + Slippage por operación (Pilar 12)
/// Usado para el cálculo de viabilidad en tiempo real.
pub const TRADING_FEE: f64 = 0.001; // 0.1%

/// Símbolos de activos (Pilar 6: Correlación Latente)
pub const MAIN_ASSET: &str = "BTCUSDT";
pub const CORR_ASSET: &str = "ETHUSDT";

/// Periodo de cálculo para el Efficiency Ratio (Pilar 3)
pub const ER_PERIOD: usize = 10;