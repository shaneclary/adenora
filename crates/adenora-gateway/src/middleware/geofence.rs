// Interface scaffolding: this whole geofencing module is retained for a planned
// per-market/lottery country-restriction feature and is not yet wired into routes.
#![allow(dead_code)]

use axum::http::HeaderMap;

/// Determine user's country from request headers.
/// In production, this uses Cloudflare's CF-IPCountry header or a GeoIP database.
/// Falls back to the user's registered country_code from their profile.
pub fn extract_country(headers: &HeaderMap) -> Option<String> {
    // Cloudflare sets this header automatically
    if let Some(cf_country) = headers.get("CF-IPCountry") {
        if let Ok(code) = cf_country.to_str() {
            let code = code.trim().to_uppercase();
            if code.len() == 2 && code != "XX" {
                return Some(code);
            }
        }
    }

    // Fallback: X-Country header (set by reverse proxy or load balancer)
    if let Some(x_country) = headers.get("X-Country") {
        if let Ok(code) = x_country.to_str() {
            let code = code.trim().to_uppercase();
            if code.len() == 2 {
                return Some(code);
            }
        }
    }

    None
}

/// Check if a user's country is allowed to access a market or lottery.
/// Markets and lotteries have country_codes arrays specifying where they're available.
/// Empty array = available everywhere.
pub fn is_country_allowed(user_country: &str, allowed_countries: &[String]) -> bool {
    if allowed_countries.is_empty() {
        return true; // No restriction = global
    }
    allowed_countries.iter().any(|c| c.eq_ignore_ascii_case(user_country))
}

/// Countries where the platform operates (GLC license countries).
pub fn platform_countries() -> Vec<&'static str> {
    vec![
        // Primary markets (Western Balkans)
        "XK", // Kosovo
        "AL", // Albania
        "MK", // North Macedonia
        // GLC license countries (Balkans + broader)
        "RS", // Serbia
        "BA", // Bosnia
        "ME", // Montenegro
        "HR", // Croatia
        "BG", // Bulgaria
        "RO", // Romania
        "HU", // Hungary
        "GR", // Greece
        "TR", // Turkey
        "MC", // Monaco
        // GLC license countries (global)
        "PH", // Philippines
        "KH", // Cambodia
        "LA", // Laos
        "VN", // Vietnam
        "CN", // China
        "RU", // Russia
        "MX", // Mexico
        "HN", // Honduras
        "NI", // Nicaragua
    ]
}

/// Check if a country is on the platform's blocked list.
/// These are countries with explicit prediction market bans.
pub fn is_blocked_country(code: &str) -> bool {
    matches!(
        code.to_uppercase().as_str(),
        "US"  // complex — state by state
        | "FR" // France
        | "DE" // Germany
        | "NL" // Netherlands
        | "BE" // Belgium
        | "PL" // Poland
        | "CH" // Switzerland
        | "SG" // Singapore
        | "TH" // Thailand
        | "GB" // UK
        | "AU" // Australia
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_country_allowed_empty_means_global() {
        assert!(is_country_allowed("XK", &[]));
        assert!(is_country_allowed("US", &[]));
    }

    #[test]
    fn test_country_allowed_restricted() {
        let allowed = vec!["XK".to_string(), "AL".to_string(), "MK".to_string()];
        assert!(is_country_allowed("XK", &allowed));
        assert!(!is_country_allowed("US", &allowed));
    }

    #[test]
    fn test_blocked_countries() {
        assert!(is_blocked_country("US"));
        assert!(is_blocked_country("FR"));
        assert!(!is_blocked_country("XK"));
        assert!(!is_blocked_country("AL"));
    }
}
