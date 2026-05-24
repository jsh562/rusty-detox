//! US4 (Library API for programmatic embedding) integration tests.

use rusty_detox::filter::Filter;
use rusty_detox::{Detox, DetoxBuilder, Sequence};
use static_assertions::assert_impl_all;
use tempfile::TempDir;

#[test]
fn send_sync_clone_bounds() {
    assert_impl_all!(Detox: Send, Sync, Clone);
    assert_impl_all!(DetoxBuilder: Send, Sync, Clone);
    assert_impl_all!(Sequence: Send, Sync, Clone);
    assert_impl_all!(Filter: Send, Sync, Clone);
}

#[test]
fn builder_default_sequence_sanitizes_space() {
    let detox = DetoxBuilder::new().build();
    assert_eq!(detox.sanitize("hello world.txt"), "hello_world.txt");
}

#[test]
fn builder_utf_8_sequence_transliterates() {
    let detox = DetoxBuilder::new().sequence(Sequence::utf_8()).build();
    assert_eq!(detox.sanitize("café.pdf"), "cafe.pdf");
}

#[test]
fn custom_sequence_with_dash_replacement() {
    // US4 AS3 — construct a custom Sequence with `-` instead of `_`.
    let seq = Sequence::new("dash-style")
        .push(Filter::Safe {
            replacement: b'-',
            unsafe_chars: b" ()[]{}<>\"'".to_vec(),
        })
        .push(Filter::Wipeup {
            separator: b'-',
            remove_trailing: false,
        });
    let detox = DetoxBuilder::new().sequence(seq).build();
    assert_eq!(detox.sanitize("hello world.txt"), "hello-world.txt");
}

#[test]
fn sanitize_bytes_parity_with_str_sc028() {
    // SC-028 — for UTF-8-clean input, sanitize(s).as_bytes() == sanitize_bytes(s.as_bytes())
    let detox = DetoxBuilder::new().sequence(Sequence::utf_8()).build();
    for input in &["ascii.txt", "café.pdf", "résumé final.docx", "clean.md"] {
        assert_eq!(
            detox.sanitize(input).as_bytes(),
            detox.sanitize_bytes(input.as_bytes()).as_slice(),
            "parity-by-construction failed for input: {input:?}"
        );
    }
}

#[test]
fn sequence_default_inherent_matches_trait_default() {
    let inherent = Sequence::default();
    let via_trait: Sequence = <Sequence as Default>::default();
    assert_eq!(inherent, via_trait);
}

#[test]
fn filter_safe_default_helper() {
    let f = Filter::safe_default();
    match f {
        Filter::Safe {
            replacement,
            unsafe_chars,
        } => {
            assert_eq!(replacement, b'_');
            assert!(unsafe_chars.contains(&b' '));
            assert!(unsafe_chars.contains(&b'/'));
            assert!(unsafe_chars.contains(&b'('));
        }
        _ => panic!("safe_default must return Filter::Safe variant"),
    }
}

#[test]
fn plan_no_filesystem_mutation() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("hello world.txt");
    std::fs::write(&src, b"contents").unwrap();

    let detox = DetoxBuilder::new().build();
    let _plan = detox.plan(&src);

    // Source unchanged.
    assert!(src.exists(), "plan() must not rename");
    assert_eq!(
        std::fs::read(&src).unwrap(),
        b"contents",
        "plan() must not modify contents"
    );
}

#[test]
fn execute_with_dry_run_skips_rename() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("hello world.txt");
    std::fs::write(&src, b"x").unwrap();

    let detox = DetoxBuilder::new().dry_run(true).build();
    let report = detox.execute(&src).unwrap();
    assert!(src.exists(), "dry-run must not rename");
    assert!(report.skipped > 0 || report.renamed == 0);
}

#[test]
fn execute_actually_renames() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("hello world.txt");
    let tgt = dir.path().join("hello_world.txt");
    std::fs::write(&src, b"x").unwrap();

    let detox = DetoxBuilder::new().build();
    let report = detox.execute(&src).unwrap();
    assert_eq!(report.renamed, 1);
    assert!(!src.exists());
    assert!(tgt.exists());
}
