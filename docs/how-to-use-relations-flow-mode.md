---
type: how-to
description: Evaluate module-defined flow rules with quire relations --flow.
---

# Inspect module-defined flows

`quire relations --flow` evaluates the `flow_rules` declared by the selected module and prints the engine's report. The CLI does not choose the rules or interpret their vocabulary. A flow denial is a finding: warning findings leave the default command successful, error findings return exit code 1, and `--strict` returns 1 for any denial.

This example uses the module and project under [`examples/flow-mode`](examples/flow-mode).

## Before you start

Install Quire CLI and make sure `quire` is on `PATH`. Run the commands from the root of the Quire CLI checkout.

## Steps

1. **Print the flow findings.**

```sh
quire relations \
  --scope "$PWD/docs/examples/flow-mode/project" \
  --module "$PWD/docs/examples/flow-mode/module" \
  --flow
```

   The module declares a `health-across-untrusted-boundary` rule. It matches `CONN-1`, which links to `CLS-1` (`regulatedCategories: [health]`) and `BOUNDARY-1` (`trust_level: untrusted`).

2. **Print the JSON report.** Add `--json` to consume the full `quire.flow-report/v1` document:

```sh
quire relations \
  --scope "$PWD/docs/examples/flow-mode/project" \
  --module "$PWD/docs/examples/flow-mode/module" \
  --flow --json
```

3. **Add the suggested control.** The remediation describes a control that protects the connection. Create a control document in the project:

```sh
cat > "$PWD/docs/examples/flow-mode/project/spec/CTRL-1.md" <<'EOF'
---
id: CTRL-1
type: Control
title: Encrypt traffic in transit
safeguards: [encrypt-in-transit]
relationships:
  - target: CONN-1
    type: protects
---
# Encrypt traffic in transit
EOF
```

4. **Recheck the flow.** Run the JSON command again. The report now has an empty `denials` array. Remove `CTRL-1.md` to restore the finding.

5. **Fail automation on any denial.** `--strict` still writes the complete report before returning exit code 1:

```sh
quire relations \
  --scope "$PWD/docs/examples/flow-mode/project" \
  --module "$PWD/docs/examples/flow-mode/module" \
  --flow --json --strict
```

## Expected output

The first command prints one warning finding with the check, subject, matched records, rationale, and `encrypt-in-transit` remediation. The JSON command writes the complete engine report to stdout. After adding `CTRL-1.md`, its `denials` array is empty. Without `--strict`, warning findings leave the command successful; with `--strict`, any denial returns exit code 1.

Without `--flow`, Quire still requires at least one `traceability.required_relations` declaration. When both declarations are present, the flow report appears beside ordinary relation rows; this sample module declares both. `--flow` can be combined with `--scope`, repeated `--repo`, repeated `--module`, `--json`, and `--strict`. It cannot be combined with `--relation` or `--design`. A module set must declare at least one `traceability.flow_rules` rule.
