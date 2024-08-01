pub type AssetId = usize;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Symbol {
    pub symbol_type: SymbolType,
    pub base_asset: AssetId,
    pub quote_asset: AssetId,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum SymbolType {
    ExchangePair,
    FuturesContract,
    Option,
}
