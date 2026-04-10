use crate::currency::Currency;
use crate::i18n::{Locale, fallback_chain};
use crate::types::*;

// === Currency tests ===

#[test]
fn test_eur_has_two_decimals() {
    assert_eq!(Currency::Eur.decimal_places(), 2);
}

#[test]
fn test_usdc_has_six_decimals() {
    assert_eq!(Currency::Usdc.decimal_places(), 6);
}

#[test]
fn test_huf_has_zero_decimals() {
    assert_eq!(Currency::Huf.decimal_places(), 0);
}

#[test]
fn test_currency_symbols() {
    assert_eq!(Currency::Eur.symbol(), "\u{20ac}");
    assert_eq!(Currency::Usd.symbol(), "$");
    assert_eq!(Currency::Usdc.symbol(), "USDC");
}

// === Locale tests ===

#[test]
fn test_launch_locales() {
    assert!(Locale::Sq.is_launch_locale());
    assert!(Locale::Mk.is_launch_locale());
    assert!(Locale::Sr.is_launch_locale());
    assert!(Locale::En.is_launch_locale());
    assert!(!Locale::Tr.is_launch_locale());
    assert!(!Locale::Zh.is_launch_locale());
}

#[test]
fn test_fallback_chain_bcms() {
    // Bosnian falls back to Serbian then English
    let chain = fallback_chain(Locale::Bs);
    assert_eq!(chain, vec![Locale::Bs, Locale::Sr, Locale::En]);
}

#[test]
fn test_fallback_chain_default() {
    let chain = fallback_chain(Locale::Sq);
    assert_eq!(chain, vec![Locale::Sq, Locale::En]);
}

#[test]
fn test_default_locale_is_english() {
    assert_eq!(Locale::default(), Locale::En);
}

#[test]
fn test_no_rtl_languages() {
    // None of our supported languages are RTL
    for locale in [Locale::Sq, Locale::Mk, Locale::Sr, Locale::En, Locale::Tr, Locale::Zh] {
        assert!(!locale.uses_rtl());
    }
}

#[test]
fn test_script_detection() {
    use crate::i18n::Script;
    assert_eq!(Locale::Sq.script(), Script::Latin);
    assert_eq!(Locale::Mk.script(), Script::Cyrillic);
    assert_eq!(Locale::El.script(), Script::Greek);
    assert_eq!(Locale::Zh.script(), Script::Han);
    assert_eq!(Locale::Km.script(), Script::Khmer);
}

// === Side tests ===

#[test]
fn test_side_opposite() {
    assert_eq!(Side::Yes.opposite(), Side::No);
    assert_eq!(Side::No.opposite(), Side::Yes);
}
