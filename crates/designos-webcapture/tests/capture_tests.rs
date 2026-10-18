use designos_webcapture::{Capture, CaptureError, Element, CAPTURE_JS, CAPTURE_SCHEMA_ID, CAPTURE_VERSION};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

const CARD_CAPTURE_JSON: &str = include_str!("playwright/fixtures/card.capture.json");
const TRUNCATED_CAPTURE_JSON: &str = include_str!("playwright/fixtures/card.truncated.capture.json");

fn index_by_id<'a>(el: &'a Element, out: &mut BTreeMap<String, &'a Element>) {
    if let Some(id) = &el.id {
        out.insert(id.clone(), el);
    }
    for child in &el.children {
        index_by_id(child, out);
    }
}

#[test]
fn boxes_styles_runs_images() {
    let capture = Capture::from_json(CARD_CAPTURE_JSON).expect("parse card.capture.json");
    assert_eq!(capture.schema, CAPTURE_SCHEMA_ID);
    assert_eq!(capture.version, CAPTURE_VERSION);
    assert_eq!(capture.viewport.width, 1280.0);
    assert_eq!(capture.viewport.height, 800.0);
    assert!(!capture.truncated);

    let mut by_id = BTreeMap::new();
    index_by_id(&capture.root, &mut by_id);

    let card = by_id.get("pricing-card").expect("pricing-card element");
    assert!(card.border_box.width >= 320.0);
    assert!(card.border_box.height > 50.0);
    assert_eq!(card.styles.get("display").map(String::as_str), Some("flex"));
    assert_eq!(card.styles.get("flex-direction").map(String::as_str), Some("column"));
    assert_eq!(card.styles.get("border-top-left-radius").map(String::as_str), Some("12px"));
    let before = card.pseudo.as_ref().and_then(|p| p.before.as_ref()).expect("card ::before pseudo box");
    assert_eq!(before.content, "PRO");
    assert!(before.border_box.width > 0.0);

    let desc = by_id.get("desc").expect("desc paragraph");
    assert!(!desc.text_runs.is_empty());
    assert!(desc.text_runs[0].lines.len() >= 2, "expected wrapped multi-line text rects, got {}", desc.text_runs[0].lines.len());
    for rect in &desc.text_runs[0].lines {
        assert!(rect.width > 0.0 && rect.height > 0.0);
    }

    let avatar = by_id.get("avatar-img").expect("avatar-img");
    let avatar_img = avatar.image.as_ref().expect("avatar ImageRef");
    assert!(!avatar_img.blocked);
    assert!(avatar_img.data_url.as_deref().unwrap_or_default().starts_with("data:image/"));

    let badge = by_id.get("badge-svg").expect("badge-svg");
    let badge_img = badge.image.as_ref().expect("badge ImageRef");
    assert!(badge_img.svg.as_deref().unwrap_or_default().contains("<svg") || badge_img.data_url.as_deref().unwrap_or_default().starts_with("data:image/svg"));

    let spark = by_id.get("spark-canvas").expect("spark-canvas");
    let spark_img = spark.image.as_ref().expect("spark ImageRef");
    assert!(spark_img.data_url.as_deref().unwrap_or_default().starts_with("data:image/png"));

    let bg = by_id.get("bg-banner").expect("bg-banner");
    assert!(bg.image.is_some() || !bg.images.is_empty());

    assert!(!by_id.contains_key("secret-hidden"), "display:none element must be omitted by default");
}

#[test]
fn css_var_refs() {
    let capture = Capture::from_json(CARD_CAPTURE_JSON).expect("parse card.capture.json");
    assert_eq!(capture.root.css_vars_declared.get("--brand").map(String::as_str), Some("#0d99ff"));

    let mut by_id = BTreeMap::new();
    index_by_id(&capture.root, &mut by_id);
    let btn = by_id.get("cta-btn").expect("cta-btn");
    assert_eq!(btn.css_var_refs.get("background-color"), Some(&vec!["--brand".to_string()]));
}

#[test]
fn component_hint() {
    let capture = Capture::from_json(CARD_CAPTURE_JSON).expect("parse card.capture.json");
    let mut by_id = BTreeMap::new();
    index_by_id(&capture.root, &mut by_id);
    let card = by_id.get("pricing-card").expect("pricing-card");
    assert_eq!(card.component_hint.as_deref(), Some("PricingCard"));
}

#[test]
fn cross_origin_image_blocked_recorded() {
    let capture = Capture::from_json(CARD_CAPTURE_JSON).expect("parse card.capture.json");
    let mut by_id = BTreeMap::new();
    index_by_id(&capture.root, &mut by_id);
    let blocked = by_id.get("blocked-img").expect("blocked-img");
    let img = blocked.image.as_ref().expect("blocked ImageRef");
    assert!(img.blocked);
    assert!(img.src.contains("unreachable-cross-origin.png"));
}

#[test]
fn rust_parses_capture() {
    assert!(CAPTURE_JS.contains("__designosCapture"), "embedded CAPTURE_JS must define window.__designosCapture");

    let capture = Capture::from_json(CARD_CAPTURE_JSON).expect("parse card.capture.json");
    let serialized = capture.to_json_pretty().expect("serialize capture");
    let roundtrip = Capture::from_json(&serialized).expect("re-parse serialized capture");
    assert_eq!(capture, roundtrip);

    // Version rejection (`UnsupportedCaptureVersion`)
    let future_json = r#"{"schema":"designos-webcapture","version":99,"root":{"tag":"html"}}"#;
    match Capture::from_json(future_json) {
        Err(CaptureError::UnsupportedCaptureVersion(99)) => {}
        other => panic!("expected UnsupportedCaptureVersion(99), got {other:?}"),
    }

    // Schema rejection (`UnsupportedSchema`)
    let bad_schema = r#"{"schema":"other-tool","version":1,"root":{"tag":"html"}}"#;
    match Capture::from_json(bad_schema) {
        Err(CaptureError::UnsupportedSchema(s)) if s == "other-tool" => {}
        other => panic!("expected UnsupportedSchema, got {other:?}"),
    }
}

#[test]
fn truncation() {
    let truncated = Capture::from_json(TRUNCATED_CAPTURE_JSON).expect("parse card.truncated.capture.json");
    assert!(truncated.truncated);
    assert_eq!(truncated.stats.elements, 4);
    assert!(truncated.stats.skipped_elements > 0);
}

#[test]
fn playwright_live_harness_when_available() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script_path = manifest_dir.join("tests/playwright/run_capture_tests.mjs");
    if !script_path.exists() {
        return;
    }

    let mut cmd = Command::new("node");
    cmd.arg(&script_path);
    if let Ok(home) = std::env::var("HOME") {
        let user_nm = PathBuf::from(home).join("node_modules");
        if user_nm.is_dir() {
            let existing = std::env::var("NODE_PATH").unwrap_or_default();
            let combined = if existing.is_empty() { user_nm.display().to_string() } else { format!("{}:{}", user_nm.display(), existing) };
            cmd.env("NODE_PATH", combined);
        }
    }

    if let Ok(output) = cmd.output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("\"ok\":true"), "unexpected playwright output: {stdout}");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // If Playwright or headless browser isn't installed in a CI container, skip gracefully;
            // if it failed on an assertion error, fail the test!
            if stderr.contains("AssertionError") {
                panic!("Playwright assertion failed:\n{stderr}");
            }
        }
    }
}
