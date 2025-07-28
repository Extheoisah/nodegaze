use crate::errors::LightningError;
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

#[derive(Deserialize)]
struct MempoolPrice {
    #[serde(rename = "USD")]
    usd: f64,
}

#[derive(Clone)]
struct PriceCache {
    price: f64,
    last_updated: SystemTime,
}

pub struct PriceConverter {
    cache: Arc<RwLock<Option<PriceCache>>>,
    client: reqwest::Client,
}

impl PriceConverter {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
            client: reqwest::Client::new(),
        }
    }

    /// Convert sats to USD (fetches BTC price internally)
    pub async fn sats_to_usd(&self, sats: u64) -> Result<f64, LightningError> {
        let btc_price = self.get_btc_price().await?;
        Ok(Self::sats_to_usd_with_price(sats, btc_price))
    }

    pub fn sats_to_usd_with_price(sats: u64, btc_price: f64) -> f64 {
        let btc_amount = sats as f64 / 100_000_000.0;
        Self::round_to_2_decimals(btc_amount * btc_price)
    }

    fn round_to_2_decimals(value: f64) -> f64 {
        (value * 100.0).round() / 100.0
    }

    /// Fetch BTC price (cached or API)
    pub async fn fetch_btc_price(&self) -> Result<f64, LightningError> {
        self.get_btc_price().await
    }

    async fn get_btc_price(&self) -> Result<f64, LightningError> {
        const CACHE_DURATION: Duration = Duration::from_secs(300); // 5 minutes

        // Try cache first (valid)
        {
            let cache = self.cache.read().await;
            if let Some(cached) = &*cache {
                if cached
                    .last_updated
                    .elapsed()
                    .is_ok_and(|e| e < CACHE_DURATION)
                {
                    return Ok(cached.price);
                }
            }
        }

        // Attempt to fetch new price
        let response = self
            .client
            .get("https://mempool.space/api/v1/prices")
            .timeout(Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) => {
                let price_data: MempoolPrice = resp
                    .json()
                    .await
                    .map_err(|e| LightningError::Parse(e.to_string()))?;

                // Update cache
                let mut cache = self.cache.write().await;
                *cache = Some(PriceCache {
                    price: price_data.usd,
                    last_updated: SystemTime::now(),
                });

                Ok(price_data.usd)
            }
            Err(_) => {
                // Fallback: use last cached price even if expired
                let cache = self.cache.read().await;
                if let Some(cached) = &*cache {
                    Ok(cached.price)
                } else {
                    Err(LightningError::NetworkError(
                        "Failed to fetch BTC price and no cached value".into(),
                    ))
                }
            }
        }
    }
}

// Global singleton instance
use std::sync::OnceLock;
static CONVERTER: OnceLock<PriceConverter> = OnceLock::new();

pub fn get_converter() -> &'static PriceConverter {
    CONVERTER.get_or_init(|| PriceConverter::new())
}
