use std::{fs::File, io::Read, path::Path, time::Duration};

use serde::Deserialize;
use thiserror::Error;

use crate::{execution::BackendMode, model::Decimal, risk::RiskLimits};

const MAX_CONFIG_BYTES: u64 = 65_536;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema_version: u8,
    pub trading_enabled: bool,
    pub backend: BackendMode,
    pub execution_queue_capacity: usize,
    pub dedupe_capacity: usize,
    pub telemetry_capacity: usize,
    pub telegram: TelegramConfig,
    pub risk: RiskConfig,
    pub strategy: StrategyConfig,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelegramConfig {
    pub channel_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RiskConfig {
    pub symbol_allowlist: Vec<String>,
    pub max_signal_age_ms: u64,
    pub max_market_age_ms: u64,
    pub max_queue_age_ms: u64,
    pub max_spread_bps: u32,
    pub max_slippage_bps: u32,
    pub max_notional: String,
    pub max_loss: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyConfig {
    pub margin_mode: Option<MarginMode>,
    pub leverage: Option<u32>,
    pub tp_alpha: Option<String>,
    pub stop_beta: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarginMode {
    Cross,
    Isolated,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid configuration syntax")]
    Syntax(#[source] toml::de::Error),
    #[error("unsupported or unsafe bootstrap configuration: {0}")]
    Invalid(&'static str),
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let mut text = String::new();
        File::open(path)?
            .take(MAX_CONFIG_BYTES + 1)
            .read_to_string(&mut text)?;
        Self::from_toml(&text)
    }

    pub fn from_toml(text: &str) -> Result<Self, ConfigError> {
        if text.len() as u64 > MAX_CONFIG_BYTES {
            return Err(ConfigError::Invalid("config too large"));
        }
        let config: Self = toml::from_str(text).map_err(ConfigError::Syntax)?;
        config.validate()?;
        Ok(config)
    }

    pub fn risk_limits(&self) -> Result<RiskLimits, ConfigError> {
        Ok(RiskLimits {
            symbol_allowlist: self.risk.symbol_allowlist.clone(),
            max_signal_age: Duration::from_millis(self.risk.max_signal_age_ms),
            max_market_age: Duration::from_millis(self.risk.max_market_age_ms),
            max_queue_age: Duration::from_millis(self.risk.max_queue_age_ms),
            max_spread_bps: self.risk.max_spread_bps,
            max_slippage_bps: self.risk.max_slippage_bps,
            max_notional: self
                .risk
                .max_notional
                .parse()
                .map_err(|_| ConfigError::Invalid("max_notional"))?,
            max_loss: self
                .risk
                .max_loss
                .parse()
                .map_err(|_| ConfigError::Invalid("max_loss"))?,
        })
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.schema_version != 1 || self.trading_enabled {
            return Err(ConfigError::Invalid(
                "schema must be 1 and trading_enabled must be false",
            ));
        }
        for capacity in [
            self.execution_queue_capacity,
            self.dedupe_capacity,
            self.telemetry_capacity,
        ] {
            if !(1..=65_536).contains(&capacity) {
                return Err(ConfigError::Invalid("capacity out of bounds"));
            }
        }
        let channels = &self.telegram.channel_ids;
        if !(channels.is_empty()
            || channels.len() == 2 && channels[0] != channels[1] && !channels.contains(&0))
        {
            return Err(ConfigError::Invalid(
                "configure zero or two distinct nonzero channel IDs",
            ));
        }
        if self.strategy.leverage == Some(0) {
            return Err(ConfigError::Invalid("zero leverage"));
        }
        for value in [&self.strategy.tp_alpha, &self.strategy.stop_beta]
            .into_iter()
            .flatten()
        {
            let decimal: Decimal = value
                .parse()
                .map_err(|_| ConfigError::Invalid("strategy fraction"))?;
            if decimal == Decimal::ZERO || decimal > Decimal::ONE {
                return Err(ConfigError::Invalid("strategy fraction outside (0,1]"));
            }
        }
        if self.risk.symbol_allowlist.len() > 256
            || self.risk.symbol_allowlist.iter().any(|s| {
                s.is_empty() || s.len() > 32 || !s.bytes().all(|b| b.is_ascii_alphanumeric())
            })
            || [
                self.risk.max_signal_age_ms,
                self.risk.max_market_age_ms,
                self.risk.max_queue_age_ms,
            ]
            .iter()
            .any(|v| *v == 0 || *v > 60_000)
            || self.risk.max_spread_bps > 10_000
            || self.risk.max_slippage_bps > 10_000
        {
            return Err(ConfigError::Invalid("risk limits out of bounds"));
        }
        self.risk_limits()?;
        Ok(())
    }
}
