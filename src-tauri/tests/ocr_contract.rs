mod support;

use support::{
    OcrFailureKind, contract_harness, sample_capture, sample_empty_capture,
    sample_missing_engine_capture,
};

#[test]
fn ocr_extracts_text_from_a_valid_capture() {
    let subject = contract_harness();
    let capture = sample_capture();

    let result = subject
        .extract_text(&capture)
        .expect("ocr should extract text from a valid capture");

    assert!(!result.text.trim().is_empty());
}

#[test]
fn ocr_reports_no_text_detected_when_the_capture_is_legible_but_empty() {
    let subject = contract_harness();
    let capture = sample_empty_capture();

    let error = subject
        .extract_text(&capture)
        .expect_err("empty captures should not be treated as engine failures");

    assert_eq!(error.kind, OcrFailureKind::NoTextDetected);
}

#[test]
fn ocr_reports_engine_unavailability_explicitly() {
    let subject = contract_harness();
    let capture = sample_missing_engine_capture();

    let error = subject
        .extract_text(&capture)
        .expect_err("missing OCR engine should surface explicitly");

    assert_eq!(error.kind, OcrFailureKind::EngineUnavailable);
    assert!(!error.message.trim().is_empty());
}
