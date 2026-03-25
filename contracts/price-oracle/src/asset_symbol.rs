use soroban_sdk::{symbol_short, Symbol};

/// Approved African market fiat symbols for oracle prices.
pub const ASSET_NGN: Symbol = symbol_short!("NGN");
pub const ASSET_KES: Symbol = symbol_short!("KES");
pub const ASSET_GHS: Symbol = symbol_short!("GHS");

/// Returns true if `asset` is one of the approved symbols (NGN, KES, GHS).
pub fn is_approved_asset_symbol(asset: Symbol) -> bool {
    asset == ASSET_NGN || asset == ASSET_KES || asset == ASSET_GHS
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::symbol_short;

    #[test]
    fn all_three_constants_are_approved() {
        assert!(is_approved_asset_symbol(ASSET_NGN));
        assert!(is_approved_asset_symbol(ASSET_KES));
        assert!(is_approved_asset_symbol(ASSET_GHS));
    }

    #[test]
    fn common_crypto_symbols_are_not_approved() {
        assert!(!is_approved_asset_symbol(symbol_short!("XLM")));
        assert!(!is_approved_asset_symbol(symbol_short!("BTC")));
    }
}
