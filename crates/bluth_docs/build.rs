use minifier::css::minify;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

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

struct Directory {
    path: PathBuf,
}

impl Directory {
    fn rust_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let entries = match fs::read_dir(&self.path) {
            Ok(entries) => entries,
            Err(_) => return files,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(Directory { path }.rust_files());
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                println!("cargo:rerun-if-changed={}", path.display());
                files.push(path);
            }
        }
        files
    }
}

struct Digest(u64);

impl Digest {
    fn of(content: &[u8]) -> Self {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Self(hasher.finish())
    }

    fn hex6(&self) -> String {
        format!("{:x}", self.0 % 0x1000000)
    }
}

struct Download {
    url: String,
}

impl Download {
    fn bytes(&self) -> Vec<u8> {
        reqwest::blocking::get(&self.url)
            .expect("Failed to download")
            .bytes()
            .expect("Failed to get bytes")
            .to_vec()
    }
}

impl File {
    fn bytes(&self) -> Vec<u8> {
        match &self.source {
            AssetSource::Url(url) => Download { url: url.clone() }.bytes(),
            AssetSource::Bytes(bytes) => bytes.clone(),
        }
    }

    fn write(&self, dir: &Path, content: &[u8]) -> (String, String, String) {
        let digest = Digest::of(content);
        let hex6 = digest.hex6();
        let filename = format!("{}.{}", self.name, self.ext);
        let url_filename = format!("{}.{}.{}", self.name, hex6, self.ext);
        let path = dir.join(&filename);
        fs::write(&path, content).expect("Failed to write asset file");
        (filename, url_filename, hex6)
    }
}

fn main() {
    println!("cargo:rerun-if-changed=src/");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let assets_dir = Path::new(&out_dir).join("assets");
    fs::create_dir_all(&assets_dir).expect("Failed to create assets directory");

    let src = Directory {
        path: PathBuf::from("src"),
    };
    let files = src.rust_files();

    let contents: Vec<String> = files
        .iter()
        .map(|path| fs::read_to_string(path).expect("Failed to read source file"))
        .collect();

    let css = encre_css::generate(
        contents.iter().map(|s| s.as_str()),
        &encre_css::Config::default(),
    );

    let css = minify(&css).expect("minification failed").to_string();

    let css_file = File {
        name: "styles".to_string(),
        source: AssetSource::Bytes(css.as_bytes().to_vec()),
        ext: "css".to_string(),
    };

    println!(
        "cargo:warning=Scanned {} source files for CSS classes",
        files.len()
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
        let content = file.bytes();
        let (filename, url_filename, hex6) = file.write(&assets_dir, &content);
        let action = match &file.source {
            AssetSource::Url(_) => "Downloaded and saved asset",
            AssetSource::Bytes(_) => "Generated asset",
        };
        println!("cargo:warning={} {}", action, filename);

        asset_paths.push((file.name.clone(), filename, url_filename, hex6));
    }

    let assets_content = asset_paths
        .into_iter()
        .map(|(name, filename, url_filename, hex6)| {
            format!(
                "pub const {}: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/assets/{}\"));\npub const {}_URL: &str = \"/assets/{}\";\npub const {}_ETAG: &str = \"{}\";",
                name.to_uppercase(),
                filename,
                name.to_uppercase(),
                url_filename,
                name.to_uppercase(),
                hex6
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let assets_file_path = assets_dir.join("generated.rs");
    fs::write(&assets_file_path, assets_content).expect("Failed to write generated.rs");
}
