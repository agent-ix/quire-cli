use ix_trace_rs::trace;
use serde_json::{json, Value};

fn complete_report() -> Value {
    let minted = quire_qualify::coverage::required_targets()
        .into_iter()
        .map(|id| json!({"id": id, "backed": true, "origin": "retained-extra-field"}))
        .collect::<Vec<_>>();
    json!({
        "minted_targets": minted,
        "unmatched_tags": [],
        "status_lies": [],
        "untracked_symbols": []
    })
}

fn check(report: &Value) -> quire_qualify::Result<String> {
    quire_qualify::coverage::check_bytes(&serde_json::to_vec(report).unwrap())
}

#[trace("TC-828", "FR-025-AC-1")]
#[test]
fn complete_backed_population_passes() {
    let observation = check(&complete_report()).expect("complete report");
    assert_eq!(
        observation,
        "assurance traceability ok: 22/22 required targets backed"
    );
}

#[trace("TC-829", "FR-025-AC-1")]
#[test]
fn missing_and_unbacked_targets_name_exact_ids() {
    let mut missing = complete_report();
    missing["minted_targets"]
        .as_array_mut()
        .unwrap()
        .retain(|entry| entry["id"] != "FR-020-AC-4");
    let error = check(&missing).expect_err("missing target must fail");
    assert!(matches!(
        error,
        quire_qualify::Error::MissingTargets { ids }
            if ids == ["FR-020-AC-4"]
    ));

    let mut unbacked = complete_report();
    let target = unbacked["minted_targets"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["id"] == "IT-141")
        .unwrap();
    target["backed"] = json!(false);
    let error = check(&unbacked).expect_err("unbacked target must fail");
    assert!(matches!(
        error,
        quire_qualify::Error::UnbackedTargets { ids } if ids == ["IT-141"]
    ));
}

#[trace("TC-830", "FR-025-AC-1")]
#[test]
fn promoted_examples_and_coverage_defects_fail_independently() {
    let mut promoted = complete_report();
    promoted["minted_targets"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": "US-006-AC-1", "backed": true}));
    assert!(matches!(
        check(&promoted).unwrap_err(),
        quire_qualify::Error::PromotedExamples { ids } if ids == ["US-006-AC-1"]
    ));

    let mut unmatched = complete_report();
    unmatched["unmatched_tags"] = json!([{"trace_id": "FR-020-AC-2"}]);
    assert!(matches!(
        check(&unmatched).unwrap_err(),
        quire_qualify::Error::UnmatchedTags { ids } if ids == ["FR-020-AC-2"]
    ));

    let mut lies = complete_report();
    lies["status_lies"] = json!([{"row_id": "R-1"}]);
    assert!(matches!(
        check(&lies).unwrap_err(),
        quire_qualify::Error::StatusLies { count: 1 }
    ));

    let mut untracked = complete_report();
    untracked["untracked_symbols"] = json!([{"symbol": "orphan"}]);
    assert!(matches!(
        check(&untracked).unwrap_err(),
        quire_qualify::Error::UntrackedSymbols { count: 1 }
    ));

    let mut unrelated = complete_report();
    unrelated["unmatched_tags"] = json!([{"trace_id": "FR-999-AC-1"}]);
    check(&unrelated).expect("unrelated unmatched tags remain outside this gate");
}

#[trace("TC-831", "FR-025-AC-2")]
#[test]
fn malformed_or_incomplete_coverage_inputs_fail_typed_deserialization() {
    for bytes in [b"{".as_slice(), b"[]".as_slice()] {
        assert!(quire_qualify::coverage::check_bytes(bytes).is_err());
    }

    for field in [
        "minted_targets",
        "unmatched_tags",
        "status_lies",
        "untracked_symbols",
    ] {
        let mut report = complete_report();
        report.as_object_mut().unwrap().remove(field);
        let error = check(&report).expect_err("missing typed field must fail");
        assert!(matches!(
            error,
            quire_qualify::Error::CoverageJson { source }
                if source.inner().to_string().contains(field)
        ));

        let mut report = complete_report();
        report[field] = json!({});
        assert!(check(&report).is_err(), "wrongly typed {field} passed");
    }

    for (field, value) in [("id", json!(7)), ("backed", json!("yes"))] {
        let mut report = complete_report();
        report["minted_targets"][0][field] = value;
        let error = check(&report).expect_err("wrongly typed target field must fail");
        assert!(matches!(
            error,
            quire_qualify::Error::CoverageJson { source }
                if source.path().to_string().ends_with(field)
        ));
    }

    let mut duplicate = complete_report();
    let repeated = duplicate["minted_targets"][0].clone();
    let duplicate_id = repeated["id"].as_str().unwrap().to_owned();
    duplicate["minted_targets"]
        .as_array_mut()
        .unwrap()
        .push(repeated);
    assert!(matches!(
        check(&duplicate).unwrap_err(),
        quire_qualify::Error::DuplicateTargets { ids } if ids == [duplicate_id]
    ));
}
