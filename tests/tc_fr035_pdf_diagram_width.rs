//! Real PDF export width checks for the default template (FR-035).

#![cfg(target_os = "linux")]

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ix_trace_rs::trace;

const WIDE: &str = "flowchart LR\n  A1[Alpha node one] --> A2[Alpha node two] --> A3[Alpha node three] --> A4[Alpha node four] --> A5[Alpha node five] --> A6[Alpha node six] --> A7[Alpha node seven] --> A8[Alpha node eight]";
const NARROW: &str = "flowchart TD\n  Solo";
const A4_WIDTH_CM: f64 = 21.0;
const SIDE_MARGINS_CM: f64 = 5.0;
const PDFIMAGE_RESOLUTION_ROUNDING_CM: f64 = 0.01;

struct Fixture {
    dir: tempfile::TempDir,
    repo: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("fixture directory");
        let repo = dir.path().join("repo");
        fs::create_dir_all(&repo).expect("repository directory");
        git(&repo, &["init", "-q", "-b", "main"]);
        git(&repo, &["config", "user.name", "Fixture"]);
        git(&repo, &["config", "user.email", "fixture@example.invalid"]);

        let document = format!(
            "---\nid: FR-021\ntitle: Diagram width fixture\nversion: \"1.0\"\ntype: FR\n---\n\
             # [FR-021] Diagram width fixture\n\n\
             ## Description\n\nA fixture for PDF diagram sizing.\n\n\
             ## Specification\n\nThe exporter SHALL size these diagrams.\n\n\
             ### Wide\n\n```mermaid\n{WIDE}\n```\n\n\
             ### Narrow\n\n```mermaid\n{NARROW}\n```\n\n\
             ## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n| FR-021-AC-1 | It draws |\n"
        );
        fs::write(repo.join("diagram.md"), document).expect("fixture document");
        git(&repo, &["add", "diagram.md"]);
        git(&repo, &["commit", "-q", "-m", "diagram fixture"]);
        Self { dir, repo }
    }

    fn work(&self) -> &Path {
        self.dir.path()
    }

    fn input(&self) -> PathBuf {
        self.repo.join("diagram.md")
    }
}

fn git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_AUTHOR_DATE", "2001-02-03T12:00:00Z")
        .env("GIT_COMMITTER_DATE", "2001-02-03T12:00:00Z")
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn require_tool(name: &str) -> PathBuf {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| panic!("this test needs `{name}` on PATH"))
}

fn png_width_and_dpi(path: &Path) -> (u64, f64) {
    let bytes = fs::read(path).expect("rendered PNG");
    assert!(
        bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "not a PNG: {}",
        path.display()
    );
    let width = u64::from(u32::from_be_bytes(
        bytes[16..20].try_into().expect("PNG IHDR width"),
    ));
    let mut dpi = 72.0;
    let mut at = 8;
    while at + 12 <= bytes.len() {
        let length =
            u32::from_be_bytes(bytes[at..at + 4].try_into().expect("PNG chunk length")) as usize;
        if &bytes[at + 4..at + 8] == b"pHYs" {
            let pixels_per_metre =
                u32::from_be_bytes(bytes[at + 8..at + 12].try_into().expect("PNG x density"));
            dpi = f64::from(pixels_per_metre) * 0.0254;
        }
        at += 12 + length;
    }
    (width, dpi)
}

#[derive(Debug)]
struct PdfImage {
    pixel_width: u64,
    horizontal_dpi: f64,
}

fn pdf_images(path: &Path) -> Vec<PdfImage> {
    let output = Command::new(require_tool("pdfimages"))
        .arg("-list")
        .arg(path)
        .output()
        .expect("pdfimages runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields.get(2) != Some(&"image") {
                return None;
            }
            Some(PdfImage {
                pixel_width: fields.get(3)?.parse().expect("image pixel width"),
                horizontal_dpi: fields.get(12)?.parse().expect("image horizontal DPI"),
            })
        })
        .collect()
}

#[trace("FR-035-AC-9")]
#[test]
fn real_pdf_scales_a_wide_diagram_to_the_text_area_and_keeps_a_narrow_one() {
    require_tool("pandoc");
    require_tool("typst");
    require_tool("mmdr");
    require_tool("engineering-assurance");

    let fixture = Fixture::new();
    let sources = [("wide", WIDE), ("narrow", NARROW)];
    let native_images: Vec<_> = sources
        .iter()
        .map(|(name, source)| {
            let source_path = fixture.work().join(format!("{name}.mmd"));
            let image_path = fixture.work().join(format!("{name}.png"));
            fs::write(&source_path, source).expect("diagram source");
            let output = Command::new(require_tool("mmdr"))
                .current_dir(fixture.work())
                .args(["-i", source_path.to_str().expect("source path"), "-o"])
                .arg(&image_path)
                .args(["-e", "png"])
                .output()
                .expect("mmdr runs");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            png_width_and_dpi(&image_path)
        })
        .collect();

    let pdf = fixture.work().join("out.pdf");
    let mut command = common::quire();
    command
        .current_dir(fixture.work())
        .arg("export")
        .arg(fixture.input())
        .args(["--to", "pdf", "--output"])
        .arg(&pdf)
        .arg("--module")
        .arg(common::validate_module());
    let output = command.output().expect("quire export runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let images = pdf_images(&pdf);
    assert_eq!(images.len(), 2, "{images:#?}");
    let text_area_cm = A4_WIDTH_CM - SIDE_MARGINS_CM;
    let width_cm = |image: &PdfImage| image.pixel_width as f64 / image.horizontal_dpi * 2.54;
    let wide_natural_cm = native_images[0].0 as f64 / native_images[0].1 * 2.54;
    assert!(
        wide_natural_cm > text_area_cm,
        "wide source is not wider than text area: {wide_natural_cm} cm"
    );
    assert!(
        width_cm(&images[0]) <= text_area_cm + PDFIMAGE_RESOLUTION_ROUNDING_CM,
        "wide PDF image is {} cm ({:?}); text area is {text_area_cm} cm",
        width_cm(&images[0]),
        images[0]
    );

    assert_eq!(
        images[1].pixel_width, native_images[1].0,
        "narrow image was resampled"
    );
    let narrow_actual_cm = width_cm(&images[1]);
    let narrow_natural_cm = native_images[1].0 as f64 / native_images[1].1 * 2.54;
    assert!(
        (narrow_actual_cm - narrow_natural_cm).abs() <= narrow_natural_cm * 0.015 + 0.01,
        "narrow PDF image is {narrow_actual_cm} cm, natural size is {narrow_natural_cm} cm"
    );
}
