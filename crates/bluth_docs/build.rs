use minifier::css::minify;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

#[derive(Debug)]
enum AssetSource {
    Url(String),
    Bytes(Vec<u8>),
}

#[derive(Debug)]
struct File {
    name: String,
    source: AssetSource,
    ext: String,
}

fn main() {
    println!("cargo:rerun-if-changed=src/");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let assets_dir = Path::new(&out_dir).join("assets");
    fs::create_dir_all(&assets_dir).expect("Failed to create assets directory");

    // Collect all Rust source files
    let source_files = collect_rust_files("src");

    // Read the content of all source files
    let source_contents: Vec<String> = source_files
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .collect();

    // Generate CSS from all source files
    let css = encre_css::generate(
        source_contents.iter().map(|s| s.as_str()),
        &encre_css::Config::default(),
    );

    let minified_css = minify(&css).expect("minification failed").to_string();

    let css_file = File {
        name: "styles".to_string(),
        source: AssetSource::Bytes(minified_css.as_bytes().to_vec()),
        ext: "css".to_string(),
    };

    println!(
        "cargo:warning=Scanned {} source files for CSS classes",
        source_files.len()
    );

    let mut files = vec![File {
        name: "datastar".to_string(),
        source: AssetSource::Url(
            "https://cdn.jsdelivr.net/gh/starfederation/datastar@v1.0.0-RC.7/bundles/datastar.js"
                .to_string(),
        ),
        ext: "js".to_string(),
    }];
    files.push(css_file);

    let mut asset_paths = Vec::new();

    for file in &files {
        let content = match &file.source {
            AssetSource::Url(url) => download(url),
            AssetSource::Bytes(bytes) => bytes.clone(),
        };
        let hash = calculate_hash(&content);
        let hash_short = format!("{:x}", hash % 0x1000000);
        let filename = format!("{}.{}", file.name, file.ext);
        let url_filename = format!("{}.{}.{}", file.name, hash_short, file.ext);
        let path = assets_dir.join(&filename);

        fs::write(&path, &content).expect("Failed to write asset file");
        let action = match &file.source {
            AssetSource::Url(_) => "Downloaded and saved asset",
            AssetSource::Bytes(_) => "Generated asset",
        };
        println!("cargo:warning={} {}", action, filename);

        asset_paths.push((file.name.clone(), filename, url_filename, hash_short));
    }

    // Generate generated.rs with constants for asset content
    let assets_content = asset_paths
        .into_iter()
        .map(|(name, filename, url_filename, hash_short)| {
            format!(
                "pub const {}: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/assets/{}\"));\npub const {}_URL: &str = \"/assets/{}\";\npub const {}_ETAG: &str = \"{}\";",
                name.to_uppercase(),
                filename,
                name.to_uppercase(),
                url_filename,
                name.to_uppercase(),
                hash_short
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let assets_file_path = assets_dir.join("generated.rs");
    fs::write(&assets_file_path, assets_content).expect("Failed to write generated.rs");
}

fn calculate_hash(content: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

fn collect_rust_files(dir: &str) -> Vec<String> {
    let mut rust_files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                if let Some(path_str) = path.to_str() {
                    rust_files.extend(collect_rust_files(path_str));
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Some(path_str) = path.to_str() {
                    rust_files.push(path_str.to_string());
                    println!("cargo:rerun-if-changed={}", path_str);
                }
            }
        }
    }

    rust_files
}

fn download(url: &str) -> Vec<u8> {
    reqwest::blocking::get(url)
        .expect("Failed to download")
        .bytes()
        .expect("Failed to get bytes")
        .to_vec()
}
