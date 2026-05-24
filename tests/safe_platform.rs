//! T073 — SafePlatform filter integration tests (Windows behavioral assertions).
//!
//! The `safe_platform` filter is unit-tested in `src/filter/safe_platform.rs`
//! against the library API directly. These integration tests exercise the
//! filter through the public `Detox` interface to confirm the rewrite rules
//! survive the full pipeline.

use rusty_detox::filter::Filter;
use rusty_detox::{DetoxBuilder, Sequence};

#[test]
fn safe_platform_rewrites_con_dot_txt() {
    let seq = Sequence::new("with-platform")
        .push(Filter::SafePlatform)
        .push(Filter::safe_default());
    let detox = DetoxBuilder::new().sequence(seq).build();
    let result = detox.sanitize("CON.txt");
    assert_eq!(result, "CON_.txt");
}

#[test]
fn safe_platform_case_insensitive_reserved_names() {
    let seq = Sequence::new("with-platform").push(Filter::SafePlatform);
    let detox = DetoxBuilder::new().sequence(seq).build();
    assert_eq!(detox.sanitize("nul"), "nul_");
    assert_eq!(detox.sanitize("Com1"), "Com1_");
    assert_eq!(detox.sanitize("LPT9.dat"), "LPT9_.dat");
}

#[test]
fn safe_platform_rewrites_reserved_chars() {
    let seq = Sequence::new("with-platform").push(Filter::SafePlatform);
    let detox = DetoxBuilder::new().sequence(seq).build();
    assert_eq!(detox.sanitize("a<b>c"), "a_b_c");
    assert_eq!(detox.sanitize("a:b|c"), "a_b_c");
    assert_eq!(detox.sanitize("a?b*c"), "a_b_c");
}

#[test]
fn safe_platform_strips_trailing_dot() {
    let seq = Sequence::new("with-platform").push(Filter::SafePlatform);
    let detox = DetoxBuilder::new().sequence(seq).build();
    assert_eq!(detox.sanitize("foo."), "foo");
    assert_eq!(detox.sanitize("bar ."), "bar");
}

#[test]
fn safe_platform_plain_name_unchanged() {
    let seq = Sequence::new("with-platform").push(Filter::SafePlatform);
    let detox = DetoxBuilder::new().sequence(seq).build();
    assert_eq!(detox.sanitize("hello.txt"), "hello.txt");
}
