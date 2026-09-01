//! IT-144: exact assurance golden bytes are consumable across languages.

use std::fs;
use std::process::Command;

use jsonschema::{Draft, JSONSchema};
use quire_rs::assurance::ASSURANCE_V1_SCHEMA;
use serde_json::Value;

const GOLDEN: &[u8] = include_bytes!("fixtures/assurance/v1.json");

// Trace: IT-144, FR-020-AC-8
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
            panic!("IT-144 requires python3 and jsonschema: {error}");
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
            panic!("IT-144 requires Node: {error}");
        }
        Err(error) => panic!("Node compatibility probe: {error}"),
    }
}
