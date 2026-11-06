//! Keep the expandable README gallery in sync with the checked-in captures.
use std::path::Path;

#[test]
fn readme_all_screenshots_gallery_contains_every_capture() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = include_str!("../README.md");
    let (_, tail) = readme
        .split_once("<summary><strong>All screenshots (41)</strong>")
        .expect("README should expose a discoverable All screenshots section");
    let (gallery, _) = tail
        .split_once("</details>")
        .expect("All screenshots section must be closed");
    let files = std::fs::read_dir(root.join("assets/screenshots"))
        .expect("screenshots directory exists")
        .map(|entry| entry.expect("valid directory entry").file_name())
        .filter(|name| name.to_string_lossy().ends_with(".png"))
        .collect::<Vec<_>>();
    assert_eq!(
        files.len(),
        41,
        "expected all historical views, designer scene and three custom-pack captures"
    );
    for file in files {
        let path = format!("assets/screenshots/{}", file.to_string_lossy());
        let clickable_preview = format!("]({path})]({path})");
        assert!(
            gallery.contains(&clickable_preview),
            "{path} must have a clickable preview inside All screenshots"
        );
    }
}
