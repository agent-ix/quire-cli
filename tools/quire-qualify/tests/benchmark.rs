use ix_trace_rs::trace;

#[trace("TC-832", "FR-025-AC-3")]
#[test]
fn nearest_rank_p95_is_deterministic_and_uses_first_result() {
    let descending = (1..=20)
        .rev()
        .map(|value| f64::from(value) / 1000.0)
        .collect::<Vec<_>>();
    let bytes = serde_json::to_vec(&serde_json::json!({
        "results": [
            {"times": descending},
            {"times": [99.0]}
        ]
    }))
    .unwrap();
    let observation = quire_qualify::benchmark::evaluate_bytes(&bytes, 50.0).unwrap();
    assert_eq!(observation.p95_ms, 19.0);
    assert_eq!(observation.sample_count, 20);
    assert_eq!(
        observation.summary(),
        "BENCH-001: p95=19.00 ms (threshold 50 ms)"
    );
}

#[trace("TC-833", "FR-025-AC-2")]
#[test]
fn malformed_empty_and_invalid_benchmark_inputs_fail_without_observation() {
    for bytes in [
        b"{".as_slice(),
        br#"{}"#.as_slice(),
        br#"{"results":null}"#.as_slice(),
        br#"{"results":[]}"#.as_slice(),
        br#"{"results":[{}]}"#.as_slice(),
        br#"{"results":[{"times":[]}] }"#.as_slice(),
        br#"{"results":[{"times":["slow"]}]}"#.as_slice(),
        br#"{"results":[{"times":[-0.1]}]}"#.as_slice(),
    ] {
        assert!(
            quire_qualify::benchmark::evaluate_bytes(bytes, 50.0).is_err(),
            "invalid input passed: {}",
            String::from_utf8_lossy(bytes)
        );
    }

    let valid = br#"{"results":[{"times":[0.01]}]}"#;
    for threshold in [-1.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(
            quire_qualify::benchmark::evaluate_bytes(valid, threshold),
            Err(quire_qualify::Error::InvalidThreshold { .. })
        ));
    }
}

#[trace("TC-834", "FR-025-AC-3")]
#[test]
fn threshold_is_inclusive_and_one_increment_over_fails() {
    let bytes = br#"{"results":[{"times":[0.05]}]}"#;
    let equal = quire_qualify::benchmark::evaluate_bytes(bytes, 50.0).unwrap();
    assert!(equal.passes());

    let over = quire_qualify::benchmark::evaluate_bytes(bytes, 49.99).unwrap();
    assert!(!over.passes());
    let error = quire_qualify::Error::BenchmarkExceeded {
        p95_ms: over.p95_ms,
        threshold_ms: over.threshold_ms,
    };
    assert_eq!(
        error.to_string(),
        "BENCH-001 failed: p95 50.00 ms > 50 ms threshold"
    );
}
