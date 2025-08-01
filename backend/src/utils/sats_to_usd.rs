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

#[derive(Clone)]
pub struct PriceConverter {
    cache: Arc<RwLock<Option<PriceCache>>>,
    client: reqwest::Client,
}

impl PriceConverter {
    const CACHE_DURATION: Duration = Duration::from_secs(120);

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
        // Check cache first (read lock)
        if let Some(cached_price) = self.check_cache().await {
            return Ok(cached_price);
        }

        // Cache miss or expired - fetch fresh price
        match self.fetch_btc_price_from_api().await {
            Ok(price) => {
                self.update_cache(price).await;
                Ok(price)
            }
            Err(e) => {
                // Fallback to stale cache if available
                self.cache
                    .read()
                    .await
                    .as_ref()
                    .map(|c| c.price)
                    .ok_or(e)
            }
        }
    }

    async fn check_cache(&self) -> Option<f64> {
        let cache = self.cache.read().await;
        cache.as_ref().and_then(|c| {
            c.last_updated
                .elapsed()
                .ok()
                .filter(|&elapsed| elapsed < Self::CACHE_DURATION)
                .map(|_| c.price)
        })
    }

    async fn fetch_btc_price_from_api(&self) -> Result<f64, LightningError> {
        let response = self
            .client
            .get("https://mempool.space/api/v1/prices")
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| LightningError::NetworkError(e.to_string()))?;

        let price_data: MempoolPrice = response
            .json()
            .await
            .map_err(|e| LightningError::Parse(e.to_string()))?;

        Ok(price_data.usd)
    }

    async fn update_cache(&self, price: f64) {
        let mut cache = self.cache.write().await;
        *cache = Some(PriceCache {
            price,
            last_updated: SystemTime::now(),
        });
    }
}