//! The generated inputs of an export (FR-034 section C, FR-035): the converter
//! metadata file, the front matter (title page and contents) and the default
//! template of each output format. Nothing here is stored as a file; all of it
//! is generated at export time into the capability root.

use anyhow::Context;
use serde_json::{Map, Value};

use super::identity::Identity;
use super::Format;

/// Backslash-escapes each ASCII punctuation character and replaces each line
/// break by a space, so a converter shows the value as literal text.
pub(super) fn escape_markdown(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    let mut leading = true;
    while let Some(character) = chars.next() {
        // Leading blanks would start an indented code block.
        if leading && matches!(character, ' ' | '\t') {
            escaped.push_str("\\ ");
            continue;
        }
        leading = false;
        match character {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                escaped.push(' ');
                leading = true;
            }
            '\n' => {
                escaped.push(' ');
                leading = true;
            }
            other if other.is_ascii_punctuation() => {
                escaped.push('\\');
                escaped.push(other);
            }
            other => escaped.push(other),
        }
    }
    escaped
}

/// The pandoc metadata file: the identity under the reserved `quire-export`
/// key as double-quoted strings (JSON is a YAML subset). The frontmatter of the
/// input is never converter input.
pub(super) fn metadata_yaml(identity: &Identity) -> anyhow::Result<String> {
    let mut export = Map::new();
    let mut put = |key: &str, value: &str| {
        export.insert(key.to_string(), Value::String(escape_markdown(value)));
    };
    put("title", &identity.title);
    put("document-id", &identity.document_id);
    if let Some(version) = &identity.version {
        put("version", version);
    }
    if let Some(release) = &identity.release {
        put("release", release);
    }
    put("commit", &identity.commit);
    put("date", &identity.date);
    let mut metadata = Map::new();
    metadata.insert("quire-export".to_string(), Value::Object(export));
    let mut yaml = serde_json::to_string(&metadata).context("encoding converter metadata")?;
    yaml.push('\n');
    Ok(yaml)
}

/// `in/front.md`: the title page, a page break, the table of contents and a
/// page break.
pub(super) fn front_matter(identity: &Identity, to: Format) -> String {
    let mut front = String::new();
    match to {
        Format::Docx => {
            front.push_str(&format!(
                "::: {{custom-style=\"Title\"}}\n{}\n:::\n\n",
                escape_markdown(&identity.title)
            ));
        }
        Format::Pdf => front.push_str(&format!("{}\n\n", escape_markdown(&identity.title))),
    }
    let mut line = |label: &str, value: &str| {
        front.push_str(&format!("{label}: {}\n\n", escape_markdown(value)));
    };
    line("Document id", &identity.document_id);
    if let Some(version) = &identity.version {
        line("Version", version);
    }
    if let Some(release) = &identity.release {
        line("Release", release);
    }
    line("Source commit", &identity.commit);
    line("Date", &identity.date);
    front.push_str(&raw_block(to, PAGE_BREAK_TYPST, PAGE_BREAK_OPENXML));
    if to == Format::Docx {
        front.push_str("::: {custom-style=\"TOC Heading\"}\nContents\n:::\n\n");
    }
    front.push_str(&raw_block(to, TOC_TYPST, TOC_OPENXML));
    front.push_str(&raw_block(to, PAGE_BREAK_TYPST, PAGE_BREAK_OPENXML));
    front
}

const PAGE_BREAK_TYPST: &str = "#pagebreak()";
const PAGE_BREAK_OPENXML: &str = r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#;
const TOC_TYPST: &str = "#outline(depth: 3)";
const TOC_OPENXML: &str = r#"<w:p><w:r><w:fldChar w:fldCharType="begin" w:dirty="true"/></w:r><w:r><w:instrText xml:space="preserve"> TOC \o "1-3" \h \z \u </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;

/// A raw block of the output format.
fn raw_block(to: Format, typst: &str, openxml: &str) -> String {
    let (kind, content) = match to {
        Format::Pdf => ("typst", typst),
        Format::Docx => ("openxml", openxml),
    };
    format!("```{{={kind}}}\n{content}\n```\n\n")
}

/// The reference text of the revision shown in headers: the version, or the
/// release when there is no version.
fn revision(identity: &Identity) -> &str {
    identity
        .version
        .as_deref()
        .or(identity.release.as_deref())
        .unwrap_or_default()
}

/// The first 12 characters of the source commit hash.
fn commit_prefix(identity: &Identity) -> &str {
    identity.commit.get(..12).unwrap_or(&identity.commit)
}

/// The default PDF template: a pandoc Typst template that reads the
/// `quire-export` metadata. The commit prefix is hexadecimal and is written
/// as literal text.
pub(super) fn typst_template(identity: &Identity) -> String {
    format!(
        r#"#let qid = [$quire-export.document-id$]
#let qrev = [$if(quire-export.version)$$quire-export.version$$else$$quire-export.release$$endif$]
#set page(
  paper: "a4",
  margin: (x: 2.5cm, y: 2.5cm),
  header: context {{
    if counter(page).get().first() > 1 [
      #qid #h(1fr) #qrev
    ]
  }},
  footer: context {{
    if counter(page).get().first() > 1 [
      {commit} #h(1fr) Page #counter(page).display() of #counter(page).final().first()
    ]
  }},
)
#set text(font: "Libertinus Serif", size: 11pt, lang: "en")
#set par(justify: false, leading: 0.65em)
#show raw: set text(font: "DejaVu Sans Mono", size: 9pt)
#show heading.where(level: 1): set text(size: 18pt)
#show heading.where(level: 2): set text(size: 14pt)
#show heading.where(level: 3): set text(size: 12pt)
#let horizontalrule = line(start: (25%, 0%), end: (75%, 0%))
#let blockquote(body) = block(inset: (left: 1em), body)
$body$
"#,
        commit = commit_prefix(identity)
    )
}

fn xml_escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            other => escaped.push(other),
        }
    }
    escaped
}

const W_NAMESPACES: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships""#;
const XML_DECLARATION: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#;
const RIGHT_TAB: &str = r#"<w:pPr><w:pStyle w:val="Header"/><w:tabs><w:tab w:val="clear" w:pos="4513"/><w:tab w:val="clear" w:pos="9026"/><w:tab w:val="right" w:pos="9071"/></w:tabs></w:pPr>"#;

fn text_run(text: &str) -> String {
    format!(
        r#"<w:r><w:t xml:space="preserve">{}</w:t></w:r>"#,
        xml_escape(text)
    )
}

fn field_run(instruction: &str) -> String {
    format!(
        r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> {instruction} </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r>"#
    )
}

fn header_part(identity: &Identity) -> String {
    format!(
        r#"{XML_DECLARATION}<w:hdr {W_NAMESPACES}><w:p>{RIGHT_TAB}{id}<w:r><w:tab/></w:r>{rev}</w:p></w:hdr>"#,
        id = text_run(&identity.document_id),
        rev = text_run(revision(identity)),
    )
}

fn footer_part(identity: &Identity) -> String {
    format!(
        r#"{XML_DECLARATION}<w:ftr {W_NAMESPACES}><w:p>{RIGHT_TAB}{commit}<w:r><w:tab/></w:r>{page_label}{page}{of}{pages}</w:p></w:ftr>"#,
        commit = text_run(commit_prefix(identity)),
        page_label = text_run("Page "),
        page = field_run("PAGE"),
        of = text_run(" of "),
        pages = field_run("NUMPAGES"),
    )
}

fn empty_header_footer(tag: &str) -> String {
    format!(
        r#"{XML_DECLARATION}<w:{tag} {W_NAMESPACES}><w:p><w:pPr><w:pStyle w:val="Header"/></w:pPr></w:p></w:{tag}>"#
    )
}

fn style(id: &str, name: &str, based_on: Option<&str>, run: &str, paragraph: &str) -> String {
    let based = based_on.map_or_else(String::new, |parent| {
        format!(r#"<w:basedOn w:val="{parent}"/>"#)
    });
    format!(
        r#"<w:style w:type="paragraph" w:styleId="{id}"><w:name w:val="{name}"/>{based}<w:qFormat/><w:pPr>{paragraph}</w:pPr><w:rPr>{run}</w:rPr></w:style>"#
    )
}

fn styles_part() -> String {
    // The names resolve through the generic families of `fontTable.xml`.
    let serif = r#"<w:rFonts w:ascii="Serif" w:hAnsi="Serif" w:cs="Serif"/>"#;
    let mono = r#"<w:rFonts w:ascii="Monospace" w:hAnsi="Monospace" w:cs="Monospace"/>"#;
    let mut styles = String::new();
    styles.push_str(&format!(
        r#"<w:docDefaults><w:rPrDefault><w:rPr>{serif}<w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="en-US"/></w:rPr></w:rPrDefault></w:docDefaults>"#
    ));
    styles.push_str(r#"<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style>"#);
    styles.push_str(&style(
        "BodyText",
        "Body Text",
        Some("Normal"),
        "",
        r#"<w:spacing w:before="180" w:after="180"/>"#,
    ));
    styles.push_str(&style(
        "FirstParagraph",
        "First Paragraph",
        Some("BodyText"),
        "",
        "",
    ));
    styles.push_str(&style(
        "Compact",
        "Compact",
        Some("BodyText"),
        "",
        r#"<w:spacing w:before="36" w:after="36"/>"#,
    ));
    styles.push_str(&style(
        "Title",
        "Title",
        Some("Normal"),
        r#"<w:b/><w:sz w:val="48"/><w:szCs w:val="48"/>"#,
        r#"<w:spacing w:before="2400" w:after="480"/>"#,
    ));
    styles.push_str(&style(
        "Subtitle",
        "Subtitle",
        Some("Title"),
        r#"<w:sz w:val="32"/>"#,
        "",
    ));
    for (level, size) in [(1, 36), (2, 28), (3, 24), (4, 22), (5, 22), (6, 22)] {
        styles.push_str(&style(
            &format!("Heading{level}"),
            &format!("heading {level}"),
            Some("Normal"),
            &format!(r#"<w:b/><w:sz w:val="{size}"/><w:szCs w:val="{size}"/>"#),
            &format!(r#"<w:keepNext/><w:spacing w:before="360" w:after="120"/><w:outlineLvl w:val="{}"/>"#, level - 1),
        ));
    }
    styles.push_str(&style(
        "TOCHeading",
        "TOC Heading",
        Some("Normal"),
        r#"<w:b/><w:sz w:val="36"/>"#,
        r#"<w:spacing w:before="240" w:after="240"/>"#,
    ));
    styles.push_str(&style(
        "BlockText",
        "Block Text",
        Some("BodyText"),
        "",
        r#"<w:ind w:left="480"/>"#,
    ));
    styles.push_str(&style(
        "SourceCode",
        "Source Code",
        Some("Normal"),
        &format!(r#"{mono}<w:sz w:val="18"/>"#),
        "",
    ));
    styles.push_str(&style(
        "FootnoteText",
        "footnote text",
        Some("Normal"),
        r#"<w:sz w:val="20"/>"#,
        "",
    ));
    styles.push_str(&style("Header", "header", Some("Normal"), r#"<w:sz w:val="18"/>"#, r#"<w:tabs><w:tab w:val="center" w:pos="4513"/><w:tab w:val="right" w:pos="9026"/></w:tabs>"#));
    styles.push_str(&style("Footer", "footer", Some("Normal"), r#"<w:sz w:val="18"/>"#, r#"<w:tabs><w:tab w:val="center" w:pos="4513"/><w:tab w:val="right" w:pos="9026"/></w:tabs>"#));
    styles.push_str(r#"<w:style w:type="character" w:default="1" w:styleId="DefaultParagraphFont"><w:name w:val="Default Paragraph Font"/><w:uiPriority w:val="1"/><w:semiHidden/></w:style>"#);
    styles.push_str(&format!(r#"<w:style w:type="character" w:styleId="VerbatimChar"><w:name w:val="Verbatim Char"/><w:rPr>{mono}<w:sz w:val="18"/></w:rPr></w:style>"#));
    styles.push_str(r#"<w:style w:type="character" w:styleId="FootnoteReference"><w:name w:val="footnote reference"/><w:rPr><w:vertAlign w:val="superscript"/></w:rPr></w:style>"#);
    styles.push_str(r#"<w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/><w:rPr><w:u w:val="single"/></w:rPr></w:style>"#);
    styles.push_str(r#"<w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/><w:tblPr><w:tblInd w:w="0" w:type="dxa"/><w:tblCellMar><w:top w:w="0" w:type="dxa"/><w:left w:w="108" w:type="dxa"/><w:bottom w:w="0" w:type="dxa"/><w:right w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style>"#);
    styles.push_str(r#"<w:style w:type="table" w:styleId="Table"><w:name w:val="Table"/><w:basedOn w:val="TableNormal"/><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="808080"/><w:left w:val="single" w:sz="4" w:space="0" w:color="808080"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="808080"/><w:right w:val="single" w:sz="4" w:space="0" w:color="808080"/><w:insideH w:val="single" w:sz="4" w:space="0" w:color="808080"/><w:insideV w:val="single" w:sz="4" w:space="0" w:color="808080"/></w:tblBorders></w:tblPr></w:style>"#);
    format!(r#"{XML_DECLARATION}<w:styles {W_NAMESPACES}>{styles}</w:styles>"#)
}

/// Two font names that only declare a generic family (serif, fixed-pitch), so
/// the document names no concrete font.
const FONT_TABLE: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:font w:name="Serif"><w:family w:val="roman"/><w:pitch w:val="variable"/></w:font><w:font w:name="Monospace"><w:family w:val="modern"/><w:pitch w:val="fixed"/></w:font></w:fonts>"#;

fn document_part() -> String {
    format!(
        r#"{XML_DECLARATION}<w:document {W_NAMESPACES}><w:body><w:p/><w:sectPr><w:headerReference w:type="default" r:id="rId10"/><w:headerReference w:type="first" r:id="rId11"/><w:footerReference w:type="default" r:id="rId12"/><w:footerReference w:type="first" r:id="rId13"/><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1417" w:right="1417" w:bottom="1417" w:left="1417" w:header="708" w:footer="708" w:gutter="0"/><w:titlePg/></w:sectPr></w:body></w:document>"#
    )
}

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/fontTable.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/><Override PartName="/word/header1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"/><Override PartName="/word/header2.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"/><Override PartName="/word/footer1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/><Override PartName="/word/footer2.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/></Types>"#;

const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

const DOCUMENT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable" Target="fontTable.xml"/><Relationship Id="rId10" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/><Relationship Id="rId11" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header2.xml"/><Relationship Id="rId12" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/><Relationship Id="rId13" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer2.xml"/></Relationships>"#;

/// A ZIP archive of stored (uncompressed) entries with the fixed timestamp
/// 1980-01-01, so the bytes do not depend on the clock.
fn stored_zip(entries: &[(&str, &[u8])]) -> anyhow::Result<Vec<u8>> {
    const DOS_DATE: u16 = (1 << 5) | 1;
    let mut archive = Vec::new();
    let mut directory = Vec::new();
    for (name, content) in entries {
        let offset = u32::try_from(archive.len()).context("reference document is too large")?;
        let size = u32::try_from(content.len()).context("reference part is too large")?;
        let name_length = u16::try_from(name.len()).context("reference part name is too long")?;
        let crc = crc32fast::hash(content);
        let mut common = Vec::new();
        common.extend_from_slice(&20u16.to_le_bytes()); // version needed
        common.extend_from_slice(&0x0800u16.to_le_bytes()); // UTF-8 names
        common.extend_from_slice(&0u16.to_le_bytes()); // stored
        common.extend_from_slice(&0u16.to_le_bytes()); // time
        common.extend_from_slice(&DOS_DATE.to_le_bytes());
        common.extend_from_slice(&crc.to_le_bytes());
        common.extend_from_slice(&size.to_le_bytes());
        common.extend_from_slice(&size.to_le_bytes());
        common.extend_from_slice(&name_length.to_le_bytes());
        common.extend_from_slice(&0u16.to_le_bytes()); // extra length
        archive.extend_from_slice(b"PK\x03\x04");
        archive.extend_from_slice(&common);
        archive.extend_from_slice(name.as_bytes());
        archive.extend_from_slice(content);
        directory.extend_from_slice(b"PK\x01\x02");
        directory.extend_from_slice(&20u16.to_le_bytes()); // version made by
        directory.extend_from_slice(&common);
        directory.extend_from_slice(&0u16.to_le_bytes()); // comment length
        directory.extend_from_slice(&0u16.to_le_bytes()); // disk number
        directory.extend_from_slice(&0u16.to_le_bytes()); // internal attributes
        directory.extend_from_slice(&0u32.to_le_bytes()); // external attributes
        directory.extend_from_slice(&offset.to_le_bytes());
        directory.extend_from_slice(name.as_bytes());
    }
    let count = u16::try_from(entries.len()).context("too many reference parts")?;
    let directory_offset =
        u32::try_from(archive.len()).context("reference document is too large")?;
    let directory_size =
        u32::try_from(directory.len()).context("reference document is too large")?;
    archive.extend_from_slice(&directory);
    archive.extend_from_slice(b"PK\x05\x06");
    archive.extend_from_slice(&0u32.to_le_bytes()); // disk numbers
    archive.extend_from_slice(&count.to_le_bytes());
    archive.extend_from_slice(&count.to_le_bytes());
    archive.extend_from_slice(&directory_size.to_le_bytes());
    archive.extend_from_slice(&directory_offset.to_le_bytes());
    archive.extend_from_slice(&0u16.to_le_bytes()); // comment length
    Ok(archive)
}

/// The default DOCX reference document: page setup, styles, and a header and
/// footer carrying the identity as literal text with `PAGE` and `NUMPAGES`
/// fields, a different first page without either.
pub(super) fn docx_reference(identity: &Identity) -> anyhow::Result<Vec<u8>> {
    let parts: [(&str, String); 10] = [
        ("[Content_Types].xml", CONTENT_TYPES.to_string()),
        ("_rels/.rels", ROOT_RELS.to_string()),
        ("word/document.xml", document_part()),
        ("word/_rels/document.xml.rels", DOCUMENT_RELS.to_string()),
        ("word/styles.xml", styles_part()),
        ("word/fontTable.xml", FONT_TABLE.to_string()),
        ("word/header1.xml", header_part(identity)),
        ("word/header2.xml", empty_header_footer("hdr")),
        ("word/footer1.xml", footer_part(identity)),
        ("word/footer2.xml", empty_header_footer("ftr")),
    ];
    let entries: Vec<(&str, &[u8])> = parts
        .iter()
        .map(|(name, content)| (*name, content.as_bytes()))
        .collect();
    stored_zip(&entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    fn identity() -> Identity {
        Identity {
            title: "*x* `#pagebreak()`{=typst}".to_string(),
            document_id: "A|B[1]".to_string(),
            version: Some("1.0".to_string()),
            release: None,
            commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
            date: "2001-02-04".to_string(),
            epoch: 0,
        }
    }

    #[test]
    #[trace("FR-034-AC-10")]
    fn markdown_punctuation_is_escaped_and_line_breaks_become_spaces() {
        assert_eq!(
            escape_markdown("*x* `a`{=typst}\r\nb\nc"),
            "\\*x\\* \\`a\\`\\{\\=typst\\} b c"
        );
    }

    #[test]
    #[trace("FR-034-AC-10")]
    fn forms_git_cannot_name_are_still_escaped_and_leading_blanks_are_kept() {
        // Characters a ref name cannot hold are still escaped when they reach a
        // value through the title or the id.
        assert_eq!(escape_markdown("*r* [a|b]"), "\\*r\\* \\[a\\|b\\]");
        assert_eq!(escape_markdown("    indented"), "\\ \\ \\ \\ indented");
        assert_eq!(escape_markdown("\tx"), "\\ x");
    }

    /// Reads a stored ZIP: returns (name, bytes) per entry after checking the
    /// local headers, the central directory, offsets and CRCs.
    fn read_stored_zip(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
        let u16_at = |at: usize| usize::from(u16::from_le_bytes([bytes[at], bytes[at + 1]]));
        let u32_at = |at: usize| {
            u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]) as usize
        };
        let end = bytes.len() - 22;
        assert_eq!(&bytes[end..end + 4], b"PK\x05\x06");
        let count = u16_at(end + 10);
        let mut at = u32_at(end + 16);
        assert_eq!(u32_at(end + 12), end - at, "central directory size");
        let mut entries = Vec::new();
        for _ in 0..count {
            assert_eq!(&bytes[at..at + 4], b"PK\x01\x02");
            let crc = u32_at(at + 16) as u32;
            let size = u32_at(at + 24);
            let name_length = u16_at(at + 28);
            let offset = u32_at(at + 42);
            let name = String::from_utf8(bytes[at + 46..at + 46 + name_length].to_vec())
                .expect("UTF-8 name");
            assert_eq!(&bytes[offset..offset + 4], b"PK\x03\x04");
            assert_eq!(u16_at(offset + 8), 0, "stored");
            assert_eq!(u32_at(offset + 14) as u32, crc);
            assert_eq!(u32_at(offset + 22), size);
            let data_at = offset + 30 + u16_at(offset + 26) + u16_at(offset + 28);
            let data = bytes[data_at..data_at + size].to_vec();
            assert_eq!(crc32fast::hash(&data), crc, "{name}");
            entries.push((name, data));
            at += 46 + name_length;
        }
        entries
    }

    #[test]
    #[trace("FR-035-AC-5")]
    fn the_reference_document_is_a_well_formed_zip_and_is_byte_stable() {
        let first = docx_reference(&identity()).expect("reference");
        let second = docx_reference(&identity()).expect("reference");
        assert_eq!(first, second);
        let entries = read_stored_zip(&first);
        let names: Vec<_> = entries.iter().map(|(name, _)| name.as_str()).collect();
        assert!(names.contains(&"[Content_Types].xml"));
        assert!(names.contains(&"word/footer1.xml"));
        assert!(names.contains(&"word/fontTable.xml"));
        for (name, data) in &entries {
            let text = std::str::from_utf8(data).expect("UTF-8 part");
            assert!(text.starts_with("<?xml"), "{name}");
        }
        let all = String::from_utf8_lossy(&first);
        assert!(!all.contains("Times New Roman") && !all.contains("Courier New"));
    }

    #[test]
    #[trace("FR-034-AC-12")]
    fn metadata_holds_escaped_double_quoted_strings_under_one_key() {
        let yaml = metadata_yaml(&identity()).expect("metadata");
        let parsed: Value = serde_json::from_str(&yaml).expect("JSON");
        assert_eq!(
            parsed["quire-export"]["document-id"],
            Value::String("A\\|B\\[1\\]".to_string())
        );
        assert!(parsed["quire-export"].get("release").is_none());
        assert_eq!(parsed.as_object().map(Map::len), Some(1));
    }

    #[test]
    #[trace("FR-035-AC-8")]
    fn front_matter_raw_blocks_match_the_output_format() {
        let pdf = front_matter(&identity(), Format::Pdf);
        assert_eq!(pdf.matches("```{=typst}").count(), 3);
        assert!(!pdf.contains("openxml"));
        let docx = front_matter(&identity(), Format::Docx);
        assert_eq!(docx.matches("```{=openxml}").count(), 3);
        assert!(docx.contains("TOC \\o \"1-3\""));
        assert!(!docx.contains("typst}"));
        assert!(pdf.contains("Source commit: 0123456789abcdef0123456789abcdef01234567"));
        assert!(!pdf.contains("Release:"));
    }

    #[test]
    #[trace("FR-035-AC-5")]
    fn the_reference_document_carries_fields_and_a_different_first_page() {
        let bytes = docx_reference(&identity()).expect("reference");
        assert_eq!(&bytes[..2], b"PK");
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("<w:titlePg/>"));
        assert!(text.contains("PAGE"));
        assert!(text.contains("NUMPAGES"));
        assert!(text.contains("0123456789ab<"));
        assert!(!text.contains("0123456789abc<"));
        assert!(text.contains("A|B[1]"));
    }
}
