---
type: how-to
description: Export a document bundle to PDF or DOCX with ordered contents and per-document sections.
---

# Export a document bundle

Use `quire export` to combine two or more Markdown specification documents into one PDF or DOCX. The input order determines the order in the contents list and the exported document.

## Before you start

Install Quire CLI and prepare at least two Markdown documents with headings. PDF export requires `pandoc`, `typst`, and `pdftotext` on `PATH`. DOCX export requires `pandoc` and `unzip` on `PATH`.

## Steps

1. **Export the bundle.** Provide the output format, bundle title, bundle ID, output path, and input files in the required order.

   ```sh
   mkdir -p build
   quire export \
     spec/functional/FR-011-lookup-subcommand.md \
     spec/functional/FR-012-edit-subcommand.md \
     spec/functional/FR-013-lint-subcommand.md \
     --to pdf \
     --bundle-title "Release Dossier" \
     --bundle-id BND-7 \
     --source-rev v0.53.1 \
     --release v0.53.1 \
     --output build/release-dossier.pdf
   ```

   `--source-rev` selects the source commit. `--release` labels the exported bundle with a tag from that source repository. This example uses the same release tag for both. For your own documents, choose a tag that contains the input files. When the tag is an ancestor of the current checkout, later commits must change files under an input directory.

   Use this command to create a Word document:

   ```sh
   quire export \
     spec/functional/FR-011-lookup-subcommand.md \
     spec/functional/FR-012-edit-subcommand.md \
     spec/functional/FR-013-lint-subcommand.md \
     --to docx \
     --bundle-title "Release Dossier" \
     --bundle-id BND-7 \
     --source-rev v0.53.1 \
     --release v0.53.1 \
     --output build/release-dossier.docx
   ```

2. **Review the output.** Check that the title page identifies the bundle, the contents list follows the input order, and each document appears in that order with a separator heading. PDF output includes page numbering after the title page. DOCX output starts each document section after a page break.

## Expected output

On success, Quire writes the requested artifact. The PDF command creates `build/release-dossier.pdf`; the DOCX command creates `build/release-dossier.docx`.

## Verify the DOCX example

To check a DOCX bundle without opening a word processor, inspect its archive and document XML:

```sh
unzip -t build/release-dossier.docx
unzip -p build/release-dossier.docx word/document.xml | grep -E 'FR-011|FR-012|FR-013'
```

## Verify with the real converters

The ordinary workspace test suite does not require the converter tools. Run the explicit real-tool suite when those tools are installed:

```sh
make test-export-real
```

The suite records converter availability and test outcomes in the generated suite evidence. The PDF bundle acceptance test also checks the title page, ordered contents, document order, and page headers and footers. The DOCX bundle acceptance test checks document order, page breaks, and heading levels.
