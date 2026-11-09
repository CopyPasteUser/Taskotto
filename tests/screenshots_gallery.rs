//! Keep the README-linked gallery and real egui capture files in sync.
use std::{collections::HashSet, fs::File, io::BufReader, path::Path};

#[test]
fn readme_linked_screenshots_gallery_contains_every_capture() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = include_str!("../README.md");
    assert!(
        readme.contains("](SCREENSHOT_GALLERY.md)"),
        "README must link the complete screenshot gallery"
    );
    let gallery = include_str!("../SCREENSHOT_GALLERY.md");
    assert!(
        gallery.contains("52 headless screenshots"),
        "gallery must describe the exact number of captured screens"
    );
    let files = std::fs::read_dir(root.join("assets/screenshots"))
        .expect("screenshots directory exists")
        .map(|entry| entry.expect("valid directory entry").file_name())
        .filter(|name| name.to_string_lossy().ends_with(".png"))
        .collect::<Vec<_>>();
    assert_eq!(
        files.len(),
        52,
        "expected all historical views, designer scene and new declarative design capture"
    );
    for file in files {
        let path = format!("assets/screenshots/{}", file.to_string_lossy());
        let clickable_preview = format!("]({path})]({path})");
        assert!(
            gallery.contains(&clickable_preview),
            "{path} must have a clickable preview in SCREENSHOT_GALLERY.md"
        );
        // Historical images vary with the live /proc workload, so exact
        // pixel hashes would be unstable. Check every committed raster is a
        // decodable, non-blank screenshot at its documented capture size.
        let decoder = png::Decoder::new(BufReader::new(File::open(root.join(&path)).unwrap()));
        let mut reader = decoder.read_info().expect("valid PNG metadata");
        let info = reader.info();
        let expected = if path.contains("diagnostic-studio-compact") {
            (720, 800)
        } else if path.contains("taskotto-win98-") {
            (650, 450)
        } else {
            (1280, 800)
        };
        assert_eq!(
            (info.width, info.height),
            expected,
            "wrong capture size: {path}"
        );
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let image = reader
            .next_frame(&mut buf)
            .expect("decodable screenshot PNG");
        let pixels = &buf[..image.buffer_size()];
        assert_eq!(
            image.color_type,
            png::ColorType::Rgba,
            "expected RGBA: {path}"
        );
        let stride = ((pixels.len() / 4) / 257).max(1);
        let variety = pixels
            .as_chunks::<4>()
            .0
            .iter()
            .step_by(stride)
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<HashSet<_>>();
        assert!(variety.len() >= 2, "screenshot appears blank: {path}");
    }
}
