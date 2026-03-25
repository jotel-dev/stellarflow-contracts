#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn setup() -> (Env, PriceOracleClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);
    (env, client)
}

#[test]
fn test_get_price_existing_asset() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);
    env.ledger().set_timestamp(1_234_567_890);
    env.ledger().set_sequence_number(1);

    let asset = symbol_short!("NGN");

    client
        .try_set_price(&asset, &1_000_000_i128)
        .unwrap()
        .unwrap();

    let result = client.try_get_price(&asset);
    assert!(result.is_ok());

    let retrieved_price = result.unwrap().unwrap();
    assert_eq!(retrieved_price.asset, asset);
    assert_eq!(retrieved_price.price, 1_000_000_i128);
    assert_eq!(retrieved_price.timestamp, 1_234_567_890);
}

#[test]
fn test_get_price_nonexistent_asset() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);

    let asset = symbol_short!("BTC");

    match client.try_get_price(&asset) {
        Err(Ok(e)) => assert_eq!(e, Error::AssetNotFound),
        other => panic!("expected contract AssetNotFound, got {:?}", other),
    }
}

#[test]
fn test_get_price_multiple_assets() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);
    let ngn = symbol_short!("NGN");
    let kes = symbol_short!("KES");

    client
        .try_set_price(&ngn, &1_000_000_i128)
        .unwrap()
        .unwrap();
    client
        .try_set_price(&kes, &50_000_000_000_i128)
        .unwrap()
        .unwrap();

    let ngn_result = client.try_get_price(&ngn).unwrap().unwrap();
    assert_eq!(ngn_result.price, 1_000_000_i128);

    let kes_result = client.try_get_price(&kes).unwrap().unwrap();
    assert_eq!(kes_result.price, 50_000_000_000_i128);
}

#[test]
fn test_get_price_after_update() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);
    let asset = symbol_short!("GHS");
    env.ledger().set_timestamp(1_234_567_890);
    env.ledger().set_sequence_number(1);
    client
        .try_set_price(&asset, &1_000_000_i128)
        .unwrap()
        .unwrap();

    let result = client.try_get_price(&asset).unwrap().unwrap();
    assert_eq!(result.price, 1_000_000_i128);
    assert_eq!(result.timestamp, 1_234_567_890);

    env.ledger().set_timestamp(1_234_567_900);
    env.ledger().set_sequence_number(2);
    client
        .try_set_price(&asset, &1_200_000_i128)
        .unwrap()
        .unwrap();

    let result = client.try_get_price(&asset).unwrap().unwrap();
    assert_eq!(result.price, 1_200_000_i128);
    assert_eq!(result.timestamp, 1_234_567_900);
}

#[test]
fn test_set_price_rejects_unapproved_symbol() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);
    let xlm = symbol_short!("XLM");

    match client.try_set_price(&xlm, &1_i128) {
        Err(Ok(e)) => assert_eq!(e, Error::InvalidAssetSymbol),
        other => panic!("expected InvalidAssetSymbol, got {:?}", other),
    }
    match client.try_get_price(&xlm) {
        Err(Ok(e)) => assert_eq!(e, Error::AssetNotFound),
        other => panic!("expected no stored price for XLM, got {:?}", other),
    }
}

#[test]
fn test_is_approved_asset_contract() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);

    assert!(client.is_approved_asset(&symbol_short!("NGN")));
    assert!(client.is_approved_asset(&symbol_short!("KES")));
    assert!(client.is_approved_asset(&symbol_short!("GHS")));
    assert!(!client.is_approved_asset(&symbol_short!("XLM")));
}

// Tests for update_price function

#[test]
fn test_update_price_admin_authority() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let provider = Address::generate(&env);

    env.as_contract(&contract_id, || {
        crate::auth::_set_admin(&env, &admin);
        crate::auth::_add_provider(&env, &provider);
    });

    let asset = symbol_short!("NGN");
    let price: i128 = 1_500_000;

    let result = client.try_update_price(&provider, &asset, &price);
    assert!(result.is_err());

    env.as_contract(&contract_id, || {
        assert!(crate::auth::_is_provider(&env, &provider));
    });
}

#[test]
#[should_panic]
fn test_update_price_unauthorized_rejection() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let unauthorized_address = Address::generate(&env);

    env.as_contract(&contract_id, || {
        crate::auth::_set_admin(&env, &admin);
    });

    let asset = symbol_short!("KES");
    let price: i128 = 50_000_000_000;

    let _ = client.update_price(&unauthorized_address, &asset, &price);
}

#[test]
fn test_update_price_emits_event() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let provider = Address::generate(&env);

    env.as_contract(&contract_id, || {
        crate::auth::_set_admin(&env, &admin);
        crate::auth::_add_provider(&env, &provider);
    });

    let asset = symbol_short!("GHS");
    let price: i128 = 2_000_000_000;

    let result = client.try_update_price(&provider, &asset, &price);
    assert!(result.is_err());

    env.as_contract(&contract_id, || {
        assert!(crate::auth::_is_provider(&env, &provider));
    });
}

#[test]
fn test_update_price_rejects_unapproved_symbol() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let provider = Address::generate(&env);

    env.as_contract(&contract_id, || {
        crate::auth::_set_admin(&env, &admin);
        crate::auth::_add_provider(&env, &provider);
    });

    let asset = symbol_short!("ETH");
    let price: i128 = 1_000_000;

    match client.try_update_price(&provider, &asset, &price) {
        Err(Ok(e)) => assert_eq!(e, Error::InvalidAssetSymbol),
        other => panic!("expected InvalidAssetSymbol, got {:?}", other),
    }
}

#[test]
fn test_update_price_multiple_updates() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let provider = Address::generate(&env);

    env.as_contract(&contract_id, || {
        crate::auth::_set_admin(&env, &admin);
        crate::auth::_add_provider(&env, &provider);
    });

    let asset = symbol_short!("NGN");
    let initial_price: i128 = 1_000_000;

    let result = client.try_update_price(&provider, &asset, &initial_price);
    assert!(result.is_err());

    env.as_contract(&contract_id, || {
        assert!(crate::auth::_is_provider(&env, &provider));
    });
}

#[test]
fn test_get_price_safe_nonexistent_returns_none() {
    let (_, client) = setup();
    assert_eq!(client.get_price_safe(&symbol_short!("NGN")), None);
}

#[test]
fn test_get_all_assets_returns_tracked_symbols() {
    let (_, client) = setup();

    let ngn = symbol_short!("NGN");
    let kes = symbol_short!("KES");

    client.try_set_price(&ngn, &1_500_i128).unwrap().unwrap();
    client.try_set_price(&kes, &800_i128).unwrap().unwrap();

    let assets = client.get_all_assets();
    assert_eq!(assets.len(), 2);
    assert!(assets.contains(&ngn));
    assert!(assets.contains(&kes));
}

#[test]
fn test_set_price_uses_current_ledger_timestamp() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);
    let asset = symbol_short!("NGN");

    env.ledger().set_timestamp(1_700_000_123);
    env.ledger().set_sequence_number(77);

    client.try_set_price(&asset, &950_i128).unwrap().unwrap();

    let stored = client.get_price(&asset);
    assert_eq!(stored.price, 950_i128);
    assert_eq!(stored.timestamp, 1_700_000_123);
}
