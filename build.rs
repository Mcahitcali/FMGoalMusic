fn main() {
    // Windows: embed app icon
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets\\app.ico");
        let _ = res.compile();
    }

    // Always attempt to rasterize SVG -> PNG for icons.
    if let Err(e) = rasterize_svg_icons() {
        // Don't fail the build; just warn so devs can fix locally.
        println!("cargo:warning=SVG rasterization failed: {e}");
    }
}

#[allow(clippy::unnecessary_wraps)]
fn rasterize_svg_icons() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;

    use resvg::{tiny_skia, usvg, FitTo};

    let svg_dir = Path::new("assets/icons/svg");
    if !svg_dir.exists() {
        // Nothing to do
        return Ok(());
    }

    // Ensure output dir exists
    fs::create_dir_all("assets/icons")?;

    // Re-run build if SVGs change
    println!("cargo:rerun-if-changed=assets/icons/svg");

    for entry in WalkDir::new(svg_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("svg") {
            continue;
        }

        let raw = fs::read(path)?;
        // Force icons to white so they look good on dark backgrounds.
        let mut svg_text = String::from_utf8(raw.clone())
            .unwrap_or_else(|_| String::from_utf8_lossy(&raw).to_string());
        svg_text = svg_text
            .replace("stroke=\"currentColor\"", "stroke=\"#ffffff\"")
            .replace("fill=\"currentColor\"", "fill=\"#ffffff\"");
        let data = svg_text.as_bytes();

        // Parse SVG
        let opt = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(data, &opt)
            .map_err(|e| format!("usvg parse error for {}: {e:?}", path.display()))?;

        // Prepare canvas
        let size = tree.size().to_int_size();
        if size.width() == 0 || size.height() == 0 {
            continue;
        }
        let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())
            .ok_or("Failed to create Pixmap")?;

        // Render at original size
        resvg::render(&tree, FitTo::Original, pixmap.as_mut())
            .ok_or("resvg render returned None")?;

        // Write PNG next to other app assets
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Bad svg filename")?;
        let out_path = PathBuf::from("assets/icons").join(format!("{stem}.png"));
        pixmap.save_png(&out_path)?;
    }

    Ok(())
}
