/// Input validation and sanitization utilities.
/// Prevents XSS, SQL injection, and other OWASP top 10 attacks.
/// Sanitize a user-provided string: strip HTML tags, limit length.
// Interface scaffolding: retained for planned input-sanitization wiring.
#[allow(dead_code)]
pub fn sanitize_text(input: &str, max_len: usize) -> String {
    input
        .chars()
        .filter(|c| !matches!(c, '<' | '>' | '&' | '"' | '\'' | '\\' | '\0'))
        .take(max_len)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Validate email format (basic check).
pub fn is_valid_email(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && domain.len() > 3
        && email.len() <= 254
        && !email.contains(' ')
}

/// Validate password strength.
pub fn is_strong_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("password must be at least 8 characters");
    }
    if password.len() > 128 {
        return Err("password must be at most 128 characters");
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("password must contain at least one uppercase letter");
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err("password must contain at least one lowercase letter");
    }
    if !password.chars().any(|c| c.is_numeric()) {
        return Err("password must contain at least one number");
    }
    Ok(())
}

/// Validate display name.
pub fn is_valid_display_name(name: &str) -> Result<(), &'static str> {
    let trimmed = name.trim();
    if trimmed.len() < 2 {
        return Err("display name must be at least 2 characters");
    }
    if trimmed.len() > 50 {
        return Err("display name must be at most 50 characters");
    }
    if trimmed.chars().any(|c| matches!(c, '<' | '>' | '&' | '"' | '\\' | '\0')) {
        return Err("display name contains invalid characters");
    }
    Ok(())
}

/// Validate ISO 3166-1 alpha-2 country code for supported countries.
// Interface scaffolding: retained for planned country-validation wiring.
#[allow(dead_code)]
pub fn is_supported_country(code: &str) -> bool {
    matches!(
        code,
        "XK" | "AL" | "MK" | "RS" | "BA" | "ME" | "HR" | "BG" | "RO" | "GR" | "TR" | "HU"
            | "CN" | "RU" | "PH" | "KH" | "LA" | "VN" | "MX" | "HN" | "NI" | "MC"
    )
}

/// Validate a prediction price (1-99 cents).
// Interface scaffolding: retained for planned price-validation wiring.
#[allow(dead_code)]
pub fn is_valid_price(cents: u32) -> bool {
    (1..=99).contains(&cents)
}

/// Validate quantity (1-10000).
// Interface scaffolding: retained for planned quantity-validation wiring.
#[allow(dead_code)]
pub fn is_valid_quantity(qty: u32) -> bool {
    (1..=10_000).contains(&qty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_strips_html() {
        assert_eq!(sanitize_text("<script>alert('xss')</script>", 100), "scriptalert(xss)/script");
    }

    #[test]
    fn test_sanitize_limits_length() {
        let long = "a".repeat(1000);
        assert_eq!(sanitize_text(&long, 50).len(), 50);
    }

    #[test]
    fn test_valid_email() {
        assert!(is_valid_email("user@example.com"));
        assert!(!is_valid_email("noatsign"));
        assert!(!is_valid_email("@nodomain"));
        assert!(!is_valid_email("user@"));
    }

    #[test]
    fn test_strong_password() {
        assert!(is_strong_password("Abc12345").is_ok());
        assert!(is_strong_password("short").is_err());
        assert!(is_strong_password("alllowercase1").is_err());
        assert!(is_strong_password("ALLUPPERCASE1").is_err());
        assert!(is_strong_password("NoNumbers!").is_err());
    }

    #[test]
    fn test_supported_countries() {
        assert!(is_supported_country("XK"));
        assert!(is_supported_country("AL"));
        assert!(!is_supported_country("US"));
        assert!(!is_supported_country("XX"));
    }
}
