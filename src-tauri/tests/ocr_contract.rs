mod support;

use support::{OcrFailureKind, phase1_harness, sample_capture};

#[test]
#[ignore = "Phase 2 will provide the OCR port and adapter"]
fn ocr_extracts_text_from_a_valid_capture() {
    let subject = phase1_harness();
    let capture = sample_capture();

    let result = subject
        .extract_text(&capture)
        .expect("ocr should extract text from a valid capture");

    assert!(!result.text.trim().is_empty());
}

#[test]
#[ignore = "Phase 2 will provide the OCR port and adapter"]
fn ocr_reports_no_text_detected_when_the_capture_is_legible_but_empty() {
    let subject = phase1_harness();
    let capture = sample_capture();

    let error = subject
        .extract_text(&capture)
        .expect_err("empty captures should not be treated as engine failures");

    assert_eq!(error.kind, OcrFailureKind::NoTextDetected);
}

#[test]
#[ignore = "Phase 2 will provide the OCR port and adapter"]
fn ocr_reports_engine_unavailability_explicitly() {
    let subject = phase1_harness();
    let capture = sample_capture();

    let error = subject
        .extract_text(&capture)
        .expect_err("missing OCR engine should surface explicitly");

    assert_eq!(error.kind, OcrFailureKind::EngineUnavailable);
    assert!(!error.message.trim().is_empty());
}
