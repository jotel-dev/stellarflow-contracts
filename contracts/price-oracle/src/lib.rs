#![no_std]
use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, symbol_short, Address, Env,
    Symbol,
};

/// Error types for the price oracle contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Asset does not exist in the price oracle
    AssetNotFound = 1,
    /// Unauthorized caller - not a whitelisted provider
    Unauthorized = 2,
    /// Asset symbol is not in the approved list (NGN, KES, GHS)
    InvalidAssetSymbol = 3,
}

/// Price data structure containing price information for an asset
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceData {
    /// The asset symbol (approved: NGN, KES, GHS)
    pub asset: Symbol,
    /// The price value (stored as scaled integer, e.g., 1000000 = 1.00 USD)
    pub price: i128,
    /// Timestamp when the price was last updated
    pub timestamp: u64,
}

/// Event emitted when a price is updated
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceUpdated {
    pub source: Address,
    pub asset: Symbol,
    pub price: i128,
    pub timestamp: u64,
}

/// Storage key for the price data map
const PRICE_DATA_KEY: Symbol = symbol_short!("PRICES");

#[contract]
pub struct PriceOracle;

#[contractimpl]
impl PriceOracle {
    /// Returns true if the symbol is approved for oracle prices (NGN, KES, GHS).
    pub fn is_approved_asset(_env: Env, asset: Symbol) -> bool {
        asset_symbol::is_approved_asset_symbol(asset)
    }

    /// Get the price data for a specific asset
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `asset` - The asset symbol to look up
    ///
    /// # Returns
    /// * `Ok(PriceData)` - The price data for the asset
    /// * `Err(Error::AssetNotFound)` - If the asset doesn't exist
    pub fn get_price(env: Env, asset: Symbol) -> Result<PriceData, Error> {
        // Get the persistent storage instance
        let storage = env.storage().persistent();

        // Try to retrieve the price data map
        let prices: soroban_sdk::Map<Symbol, PriceData> = storage
            .get(&PRICE_DATA_KEY)
            .unwrap_or_else(|| soroban_sdk::Map::new(&env));

        // Try to get the price for the specified asset
        match prices.get(asset) {
            Some(price_data) => Ok(price_data),
            None => Err(Error::AssetNotFound),
        }
    }

    /// Returns None instead of an error when asset is not found — safe for frontend callers.
    pub fn get_price_safe(env: Env, asset: Symbol) -> Option<PriceData> {
        let prices: soroban_sdk::Map<Symbol, PriceData> = env
            .storage()
            .persistent()
            .get(&PRICE_DATA_KEY)
            .unwrap_or_else(|| soroban_sdk::Map::new(&env));
        prices.get(asset)
    }

    /// Returns a Vec of all currently tracked asset symbols.
    pub fn get_all_assets(env: Env) -> soroban_sdk::Vec<Symbol> {
        let prices: soroban_sdk::Map<Symbol, PriceData> = env
            .storage()
            .persistent()
            .get(&PRICE_DATA_KEY)
            .unwrap_or_else(|| soroban_sdk::Map::new(&env));
        prices.keys()
    }

    /// Set the price data for a specific asset (admin function)
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `asset` - The asset symbol
    /// * `val` - The price value to store
    ///
    /// # Errors
    /// * `Error::InvalidAssetSymbol` - If `asset` is not NGN, KES, or GHS
    pub fn set_price(env: Env, asset: Symbol, val: i128) -> Result<(), Error> {
        if !asset_symbol::is_approved_asset_symbol(asset.clone()) {
            return Err(Error::InvalidAssetSymbol);
        }

        let storage = env.storage().persistent();

        let mut prices: soroban_sdk::Map<Symbol, PriceData> = storage
            .get(&PRICE_DATA_KEY)
            .unwrap_or_else(|| soroban_sdk::Map::new(&env));

        let price_data = PriceData {
            asset: asset.clone(),
            price: val,
            timestamp: env.ledger().timestamp(),
        };

        prices.set(asset, price_data);
        storage.set(&PRICE_DATA_KEY, &prices);
        Ok(())
    }

    /// Update the price for a specific asset (authorized backend relayer function)
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `source` - The address of the authorized backend relayer
    /// * `asset` - The asset symbol to update
    /// * `price` - The new price (as i128)
    ///
    /// # Errors
    /// * `Error::InvalidAssetSymbol` - If `asset` is not NGN, KES, or GHS
    ///
    /// # Panics
    /// If `source` is not a whitelisted provider.
    pub fn update_price(env: Env, source: Address, asset: Symbol, price: i128) -> Result<(), Error> {
        if !asset_symbol::is_approved_asset_symbol(asset.clone()) {
            return Err(Error::InvalidAssetSymbol);
        }

        if !crate::auth::_is_provider(&env, &source) {
            panic!("Unauthorised: caller is not a whitelisted provider");
        }

        source.require_auth();

        let storage = env.storage().instance();

        let mut prices: soroban_sdk::Map<Symbol, PriceData> = storage
            .get(&PRICE_DATA_KEY)
            .unwrap_or_else(|| soroban_sdk::Map::new(&env));

        let timestamp = env.ledger().timestamp();

        let price_data = PriceData {
            asset: asset.clone(),
            price,
            timestamp,
        };

        prices.set(asset.clone(), price_data);
        storage.set(&PRICE_DATA_KEY, &prices);

        PriceUpdated {
            source: source.clone(),
            asset: asset.clone(),
            price,
            timestamp,
        }
        .publish(&env);

        Ok(())
    }
}

mod asset_symbol;
mod auth;
mod median;
mod test;
