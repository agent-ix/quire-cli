//! Exact assurance golden bytes are consumable across languages.

use ix_trace_rs::trace;
use std::fs;
use std::process::Command;

use jsonschema::{Draft, JSONSchema};
use quire_rs::assurance::ASSURANCE_V1_SCHEMA;
use quire_rs::{read_assurance_export, AcceptedAssurancePremises, AssuranceExport};
use serde_json::Value;

const GOLDEN: &[u8] = include_bytes!("fixtures/assurance/v1.json");

#[trace("FR-020-AC-8")]
#[test]
fn it_144_rust_python_and_node_consume_the_exact_golden_bytes() {
    let payload: Value = serde_json::from_slice(GOLDEN).expect("golden JSON");
    let schema: Value = serde_json::from_str(ASSURANCE_V1_SCHEMA).expect("upstream schema");
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
        .expect("schema compiles");
    compiled
        .validate(&payload)
        .map_err(|errors| errors.map(|error| error.to_string()).collect::<Vec<_>>())
        .expect("Rust validates the exact golden");

    let temp = tempfile::tempdir().expect("tempdir");
    let fixture_path = temp.path().join("v1.json");
    let schema_path = temp.path().join("assurance-v1.schema.json");
    fs::write(&fixture_path, GOLDEN).expect("fixture bytes");
    fs::write(&schema_path, ASSURANCE_V1_SCHEMA).expect("schema bytes");

    match Command::new("python3")
        .arg("-c")
        .arg("import json, jsonschema, pathlib, sys; p=pathlib.Path(sys.argv[1]).read_bytes(); s=pathlib.Path(sys.argv[2]).read_bytes(); v=json.loads(p); jsonschema.validate(v, json.loads(s)); assert v['format']=='quire-assurance'; assert b'\\n' not in p[:-1]")
        .arg(&fixture_path)
        .arg(&schema_path)
        .output()
    {
        Ok(output) if output.status.success() => {}
        Ok(output) => panic!("Python compatibility probe failed: {}", String::from_utf8_lossy(&output.stderr)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            panic!("cross-language check requires python3 and jsonschema: {error}");
        }
        Err(error) => panic!("Python compatibility probe: {error}"),
    }

    match Command::new("node")
        .arg("-e")
        .arg("const fs=require('fs'); const b=fs.readFileSync(process.argv[1]); const v=JSON.parse(b); if(v.format!=='quire-assurance'||v.format_version!==1||!Array.isArray(v.relation_observations)) process.exit(2); if(b.subarray(0,b.length-1).includes(10)) process.exit(3);")
        .arg(&fixture_path)
        .output()
    {
        Ok(output) if output.status.success() => {}
        Ok(output) => panic!("Node compatibility probe failed: {}", String::from_utf8_lossy(&output.stderr)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            panic!("cross-language check requires Node: {error}");
        }
        Err(error) => panic!("Node compatibility probe: {error}"),
    }
}

#[trace("FR-020-AC-8")]
#[test]
fn it_144_languages_receive_one_acceptance_refusal_corpus() {
    let mut valid: Value = serde_json::from_slice(GOLDEN).expect("golden JSON");
    let typed: AssuranceExport = serde_json::from_value(valid.clone()).expect("typed golden");
    let accepted = AcceptedAssurancePremises::from_export(&typed);
    let mut cases = vec![reader_case(
        "valid-v1",
        serde_json::to_vec(&valid).unwrap(),
        true,
        true,
    )];

    let mut unsupported_version = valid.clone();
    unsupported_version["format_version"] = 2.into();
    cases.push(reader_case(
        "unknown-format-version",
        serde_json::to_vec(&unsupported_version).unwrap(),
        false,
        true,
    ));

    let mut duplicate_module = valid.clone();
    let repeated_module = duplicate_module["modules"][0].clone();
    duplicate_module["modules"]
        .as_array_mut()
        .unwrap()
        .push(repeated_module);
    cases.push(reader_case(
        "duplicate-module-identity",
        serde_json::to_vec(&duplicate_module).unwrap(),
        false,
        true,
    ));

    let mut duplicate_schema = valid.clone();
    let repeated_schema = duplicate_schema["modules"][0]["schemas"][0].clone();
    duplicate_schema["modules"][0]["schemas"]
        .as_array_mut()
        .unwrap()
        .push(repeated_schema);
    cases.push(reader_case(
        "duplicate-schema-identity",
        serde_json::to_vec(&duplicate_schema).unwrap(),
        false,
        true,
    ));

    let mut inconsistent_module = valid.clone();
    inconsistent_module["modules"][0]["version"] = "9.9.9".into();
    cases.push(reader_case(
        "inconsistent-module-premise",
        serde_json::to_vec(&inconsistent_module).unwrap(),
        false,
        true,
    ));

    let mut unknown_field = valid.clone();
    unknown_field["unexpected"] = true.into();
    cases.push(reader_case(
        "closed-envelope-refuses-unknown-field",
        serde_json::to_vec(&unknown_field).unwrap(),
        false,
        true,
    ));

    cases.push(reader_case("malformed-json", b"{".to_vec(), false, false));
    let mut trailing_garbage = GOLDEN.to_vec();
    trailing_garbage.extend_from_slice(b" garbage");
    cases.push(reader_case(
        "trailing-garbage",
        trailing_garbage,
        false,
        false,
    ));
    let mut concatenated = GOLDEN.to_vec();
    concatenated.extend_from_slice(GOLDEN);
    cases.push(reader_case("concatenated-json", concatenated, false, false));

    let artifacts = valid["artifacts"].as_array_mut().expect("artifact array");
    for index in 0..512 {
        artifacts.push(serde_json::json!({
            "id": format!("bounded-artifact-{index:04}"),
            "artifact_type": "FR",
            "locator": {"path": "spec/FR-001.md", "line": 1}
        }));
    }
    cases.push(reader_case(
        "bounded-large-valid-export",
        serde_json::to_vec(&valid).unwrap(),
        true,
        true,
    ));

    let mut deep = serde_json::json!(0);
    for _ in 0..96 {
        deep = serde_json::json!([deep]);
    }
    let mut deeply_nested = serde_json::from_slice::<Value>(GOLDEN).unwrap();
    deeply_nested["unknown_nested"] = deep;
    cases.push(reader_case(
        "bounded-deep-unknown-value",
        serde_json::to_vec(&deeply_nested).unwrap(),
        false,
        true,
    ));

    for case in &cases {
        let bytes = case["input"].as_str().unwrap().as_bytes();
        let expected = case["accepted"].as_bool().unwrap();
        assert_eq!(
            read_assurance_export(bytes, &accepted).is_ok(),
            expected,
            "Rust reader outcome for {}",
            case["name"]
        );
    }

    let temp = tempfile::tempdir().expect("tempdir");
    let corpus_path = temp.path().join("reader-cases.json");
    let schema_path = temp.path().join("assurance-v1.schema.json");
    fs::write(&corpus_path, serde_json::to_vec(&cases).unwrap()).expect("shared case corpus");
    fs::write(&schema_path, ASSURANCE_V1_SCHEMA).expect("schema bytes");

    let python = r#"
import json, pathlib, sys, jsonschema
cases = json.loads(pathlib.Path(sys.argv[1]).read_bytes())
schema = json.loads(pathlib.Path(sys.argv[2]).read_bytes())
accepted = json.loads(pathlib.Path(sys.argv[3]).read_bytes())
for case in cases:
    actual = False
    try:
        value = json.loads(case['input'])
        jsonschema.validate(value, schema)
        if value['format_version'] != 1:
            raise ValueError('unsupported version')
        modules = value['modules']
        names = [module['name'] for module in modules]
        if len(names) != len(set(names)):
            raise ValueError('duplicate module identity')
        for module in modules:
            archetypes = [item['archetype'] for item in module['schemas']]
            if len(archetypes) != len(set(archetypes)):
                raise ValueError('duplicate schema identity')
        if modules != accepted['modules']:
            raise ValueError('unaccepted module premise')
        actual = True
    except (ValueError, KeyError, TypeError, json.JSONDecodeError, jsonschema.ValidationError):
        actual = False
    if actual != case['accepted']:
        raise AssertionError(f"Python reader {case['name']}: {actual} != {case['accepted']}")
"#;
    let accepted_path = temp.path().join("accepted-premises.json");
    fs::write(
        &accepted_path,
        serde_json::to_vec(&serde_json::json!({"modules": typed.modules})).unwrap(),
    )
    .expect("accepted premises");
    let output = Command::new("python3")
        .arg("-c")
        .arg(python)
        .arg(&corpus_path)
        .arg(&schema_path)
        .arg(&accepted_path)
        .output()
        .expect("Python reader runtime is required");
    assert!(
        output.status.success(),
        "Python acceptance/refusal corpus failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Node is a JSON syntax/runtime probe only: quire-cli owns no Node assurance
    // schema reader or schema-validation dependency on this supported path.
    let node = r#"
const fs = require('fs');
const cases = JSON.parse(fs.readFileSync(process.argv[1], 'utf8'));
for (const testCase of cases) {
  let parsed = true;
  try { JSON.parse(testCase.input); } catch (_) { parsed = false; }
  if (parsed !== testCase.json_syntax_valid) process.exit(2);
}
"#;
    let output = Command::new("node")
        .arg("-e")
        .arg(node)
        .arg(&corpus_path)
        .output()
        .expect("Node runtime is required");
    assert!(
        output.status.success(),
        "Node JSON syntax probe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn reader_case(name: &str, input: Vec<u8>, accepted: bool, json_syntax_valid: bool) -> Value {
    serde_json::json!({
        "name": name,
        "input": String::from_utf8(input).expect("assurance reader cases are UTF-8"),
        "accepted": accepted,
        "json_syntax_valid": json_syntax_valid
    })
}
