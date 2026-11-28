---
title: Export a coverage matrix
type: how-to
description: Export a JSON coverage matrix to Word or PDF.
---

Export a JSON coverage matrix to Word or PDF with `quire export`.

## Before you start

Install Quire CLI 0.53.5 and make `quire` available on `PATH`. Run these commands from a directory where you can write `build/`. PDF export needs Pandoc and Typst. DOCX export needs Pandoc.

## Steps

1. **Save the matrix JSON.** Set the repository and module paths for the matrix.

   ```sh
   REPO=/path/to/repository
   MODULE=/path/to/traceability-module
   mkdir -p build
   quire matrix --scope "$REPO" --module "$MODULE" --format json > build/matrix.json
   ```

2. **Export the matrix.** The `matrix` kind renders matrix groups and rows as a report.

   ```sh
   quire export build/matrix.json \
     --kind matrix \
     --to docx \
     --output build/matrix.docx
   ```

   Replace `docx` with `pdf` to create a PDF. The input JSON stays unchanged.

3. **Check the DOCX file.** Verify that the output archive is readable.

   ```sh
   unzip -t build/matrix.docx
   ```

## Result

The DOCX contains the matrix groups and criterion rows from the JSON report.

## Next steps

- [Export a document bundle](how-to-export-a-document-bundle.md)
- [Export Mermaid diagrams](how-to-export-mermaid-diagrams.md)
