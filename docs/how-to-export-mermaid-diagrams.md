---
type: how-to
description: Export Markdown diagrams as images in PDF or DOCX documents.
---

# Export Mermaid diagrams as images

Use Quire CLI to export Markdown documents with Mermaid diagrams as PDF or DOCX files. The export renders each diagram as an image inside the document.

## Before you start

Install Quoin, Quire CLI, `mmdr`, and Pandoc. PDF export also needs Typst. Keep the Markdown input committed in a Git repository because Quire reads it from the source commit.

Install the default artifact modules if Quire has not installed them yet:

```sh
quoin module ensure-defaults
```

Run these commands from the Quire CLI repository root. The example input is [the device check flow](examples/mermaid-flowchart.md).

## Steps

1. **Export a PDF.** The command writes the rendered diagram into the PDF.

   ```sh
   mkdir -p build
   quire export docs/examples/mermaid-flowchart.md \
     --to pdf \
     --source-rev HEAD \
     --output build/device-check.pdf
   ```

2. **Export a DOCX file.** Use the same input and source revision.

   ```sh
   quire export docs/examples/mermaid-flowchart.md \
     --to docx \
     --source-rev HEAD \
     --output build/device-check.docx
   ```

## Expected output

Quire writes `build/device-check.pdf` and `build/device-check.docx` when both exports succeed. The diagram appears as an image in each file.

## Verify the images

List the image embedded in the PDF and the image stored in the DOCX archive:

```sh
pdfimages -list build/device-check.pdf
unzip -l build/device-check.docx | grep 'word/media/'
```

The PDF image list and DOCX archive each contain a PNG for the flowchart.
