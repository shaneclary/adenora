use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    /// Euro — used in Kosovo, standard for EU
    Eur,
    /// Albanian Lek
    All,
    /// Macedonian Denar
    Mkd,
    /// Serbian Dinar
    Rsd,
    /// Bosnian Convertible Mark
    Bam,
    /// Croatian Kuna (now EUR but kept for legacy)
    Hrk,
    /// Bulgarian Lev
    Bgn,
    /// Romanian Leu
    Ron,
    /// Hungarian Forint
    Huf,
    /// Turkish Lira
    Try,
    /// US Dollar
    Usd,
    /// USDC stablecoin
    Usdc,
}

impl Currency {
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::Eur => "\u{20ac}",
            Currency::All => "L",
            Currency::Mkd => "den",
            Currency::Rsd => "din",
            Currency::Bam => "KM",
            Currency::Hrk => "kn",
            Currency::Bgn => "лв",
            Currency::Ron => "lei",
            Currency::Huf => "Ft",
            Currency::Try => "\u{20ba}",
            Currency::Usd => "$",
            Currency::Usdc => "USDC",
        }
    }

    /// Parse an ISO-style currency code (case-insensitive). Returns `None` for
    /// codes the platform does not support.
    pub fn from_code(code: &str) -> Option<Currency> {
        match code.to_ascii_uppercase().as_str() {
            "EUR" => Some(Currency::Eur),
            "ALL" => Some(Currency::All),
            "MKD" => Some(Currency::Mkd),
            "RSD" => Some(Currency::Rsd),
            "BAM" => Some(Currency::Bam),
            "HRK" => Some(Currency::Hrk),
            "BGN" => Some(Currency::Bgn),
            "RON" => Some(Currency::Ron),
            "HUF" => Some(Currency::Huf),
            "TRY" => Some(Currency::Try),
            "USD" => Some(Currency::Usd),
            "USDC" => Some(Currency::Usdc),
            _ => None,
        }
    }

    pub fn decimal_places(&self) -> u32 {
        match self {
            Currency::Usdc => 6, // USDC uses 6 decimals on Polygon
            Currency::Huf => 0,  // Forint has no subunits
            _ => 2,
        }
    }

    pub fn smallest_unit(&self) -> Decimal {
        let places = self.decimal_places();
        Decimal::new(1, places)
    }
}
