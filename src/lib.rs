pub mod fetch;
pub mod parse;
pub mod store;

use chrono::NaiveDateTime;

/// One currency's row from the rate sheet, rates kept as published strings.
#[derive(Debug, Clone, PartialEq)]
pub struct CurrencyRates {
    pub currency: String,
    /// TT BUY, TT SELL, BILL BUY, BILL SELL, FOREX TRAVEL CARD BUY,
    /// FOREX TRAVEL CARD SELL, CN BUY, CN SELL
    pub rates: Vec<String>,
}

/// A fully parsed daily rate sheet.
#[derive(Debug, Clone)]
pub struct RateSheet {
    pub published_at: NaiveDateTime,
    pub rates: Vec<CurrencyRates>,
}

pub const RATE_COLUMNS: [&str; 8] = [
    "TT BUY",
    "TT SELL",
    "BILL BUY",
    "BILL SELL",
    "FOREX TRAVEL CARD BUY",
    "FOREX TRAVEL CARD SELL",
    "CN BUY",
    "CN SELL",
];

pub const CSV_DATE_FORMAT: &str = "%Y-%m-%d %H:%M";
