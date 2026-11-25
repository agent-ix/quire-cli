---
id: IU-001
title: Quire CLI intended use
type: IntendedUse
tool: quire-cli
relationships: []
---
# Quire CLI intended use

## Purpose

Quire CLI provides a local command interface for working with engineering
specifications and related records. It parses and extracts document data,
validates documents against their declared structures, reports traceability
and coverage information, and prepares selected assurance exports. It also
provides bounded developer checks for repository metadata and qualification
inputs.

## Records produced

| Record kind | Command that writes it |
|---|---|
| Parsed document data and extracted records, emitted as JSON or diagnostics | `quire parse`, `quire extract`, `quire lookup`, `quire schema` |
| Validation, traceability, coverage, assurance, and developer-check observations, emitted to standard output and diagnostics | `quire validate`, `quire coverage`, `quire matrix`, `quire relations`, `quire symbols`, `quire trace`, `quire properties`, `quire assurance`, `quire clauses`, `quire check-lock`, `quire publish-check`, `quire selftest --run` |
| Markdown documents updated in place | `quire edit`, `quire fix --write` |
| PDF or DOCX export files | `quire export --output` |

## Who relies on them

Specification authors and repository maintainers use the command outputs to
inspect documents, find structural and traceability issues, examine reported
coverage, and prepare or update selected engineering records. A downstream
reviewer may use an exported document or retained command output as input to a
separate review or validation process.

## Limits

Quire CLI is not a compliance oracle. Its output does not establish that a
repository, process, product, or clinical use is compliant, safe, effective,
or accepted. It does not make product or clinical safety decisions, assign
feature-risk ratings, or determine that a tool has been validated. Reports
depend on the input documents, module definitions, command options, and
available external tools. A person remains responsible for interpreting the
results and accepting any risk rating or validation conclusion.

## Operating context

Quire CLI runs as a local command-line program. Inputs may include Markdown
specification documents, JSON records, module manifests and schemas, source
repository metadata, Cargo lockfiles, and qualification data. It reads and
writes files only as requested by the selected command, and some report
commands emit observations only to standard output or diagnostics.

The CLI links the quire-rs engine. Depending on the selected operation and
available configuration, it may invoke `quoin` to install default modules,
`git` to identify a repository for validation, or
`engineering-assurance` for export production; that export process may use
`pandoc`, `typst`, or `mmdr`. The self-test runner uses the operating system's
process and filesystem controls and the external executables declared by its
selected cases. Running cases is supported on Linux; other operating systems
are rejected before a case starts. It does not require a source checkout or
network access to run embedded cases.
