use backend::utils::sats_to_usd::PriceConverter;

#[test]
fn test_sats_to_usd_with_price() {
    assert_eq!(
        PriceConverter::sats_to_usd_with_price(100_000_000, 50_000.0),
        50_000.0
    );
    assert_eq!(
        PriceConverter::sats_to_usd_with_price(50_000_000, 50_000.0),
        25_000.0
    );
    assert_eq!(PriceConverter::sats_to_usd_with_price(1_000, 50_000.0), 0.5);
    assert_eq!(PriceConverter::sats_to_usd_with_price(0, 50_000.0), 0.0);
}

#[test]
fn test_round_to_2_decimals() {
    assert_eq!(PriceConverter::round_to_2_decimals(123.456), 123.46);
    assert_eq!(PriceConverter::round_to_2_decimals(123.454), 123.45);
    assert_eq!(PriceConverter::round_to_2_decimals(99.999), 100.0);
}

#[tokio::test]
async fn test_cache_update_and_retrieval() {
    let converter = PriceConverter::new();
    converter.update_cache(50_000.0).await;

    assert_eq!(converter.check_cache().await, Some(50_000.0));
}

#[tokio::test]
async fn test_sats_to_usd_with_cached_price() {
    let converter = PriceConverter::new();
    converter.update_cache(60_000.0).await;

    let result = converter.sats_to_usd(100_000_000).await.unwrap();
    assert_eq!(result, 60_000.0);
}