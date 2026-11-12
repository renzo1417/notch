use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const INPUTS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "heic", "heif", "tif", "tiff", "bmp", "gif", "avif", "pdf",
    "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "md", "csv",
];
pub const TARGETS: &[&str] = &[
    "jpg", "png", "webp", "pdf", "docx", "xlsx", "csv", "txt", "html",
];
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn identity() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{now:x}-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionFile {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub bytes: u64,
    pub state: String,
    pub progress: Option<f64>,
    pub outputs: Vec<String>,
    pub error: Option<String>,
}

pub fn inspect(paths: Vec<String>) -> Result<Vec<ConversionFile>, String> {
    if paths.is_empty() || paths.len() > 20 {
        return Err("Choose between 1 and 20 files.".into());
    }
    let mut seen = HashSet::new();
    let mut files = Vec::new();
    for input in paths {
        if !Path::new(&input).is_absolute() {
            return Err("Choose a local file using an absolute path.".into());
        }
        let path = fs::canonicalize(&input)
            .map_err(|_| format!("The file is no longer available: {input}"))?;
        let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
        if !metadata.is_file() {
            return Err("Drop individual files, rather than folders.".into());
        }
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !INPUTS.contains(&extension.as_str()) {
            return Err(format!(
                "The .{extension} format is not supported in this version."
            ));
        }
        let text = path.to_string_lossy().into_owned();
        if !seen.insert(text.to_lowercase()) {
            continue;
        }
        files.push(ConversionFile {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            path: text,
            extension,
            bytes: metadata.len(),
            state: "pending".into(),
            progress: None,
            outputs: Vec::new(),
            error: None,
        });
    }
    Ok(files)
}

/// A directory created by this job. Cleanup never traverses a reparse point.
pub struct Staging {
    pub path: PathBuf,
    keep: bool,
}
impl Staging {
    pub fn new(parent: &Path) -> Result<Self, String> {
        fs::create_dir_all(parent).map_err(|_| {
            "Could not create the output folder. Choose another destination.".to_string()
        })?;
        let path = parent.join(format!(".notch-convert-{}", identity()));
        fs::create_dir(&path).map_err(|e| e.to_string())?;
        Ok(Self {
            path: fs::canonicalize(path).map_err(|e| e.to_string())?,
            keep: false,
        })
    }

    pub fn publish(
        &self,
        source: &str,
        target: &str,
        outputs: &[String],
    ) -> Result<Vec<String>, String> {
        if outputs.is_empty() {
            return Err("Filee reported success without producing a file.".into());
        }
        validate_tree(&self.path, 0, &mut 0)?;
        let mut relative = Vec::new();
        for output in outputs {
            let path =
                fs::canonicalize(output).map_err(|_| "Filee's output is missing.".to_string())?;
            if !path.is_file() {
                return Err("Filee returned an invalid output file.".into());
            }
            relative.push(
                path.strip_prefix(&self.path)
                    .map_err(|_| "Filee wrote outside this job's output folder.".to_string())?
                    .to_path_buf(),
            );
        }
        let stem = Path::new(source)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let stem: String = stem.chars().take(100).collect();
        let parent = self
            .path
            .parent()
            .ok_or("The output folder is unavailable.")?;
        // Reserve the result folder atomically; never replace an existing user file.
        let mut destination = None;
        for number in 0..10_000 {
            let suffix = if number == 0 {
                String::new()
            } else {
                format!(" ({number})")
            };
            let candidate = parent.join(format!("{stem}-{target}{suffix}"));
            match fs::create_dir(&candidate) {
                Ok(()) => {
                    destination = Some(candidate);
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => {
                    return Err("Could not save the result. Choose another output folder.".into())
                }
            }
        }
        let destination = destination.ok_or("Too many files share this result name.")?;
        let mut published = Staging {
            path: destination.clone(),
            keep: false,
        };
        for entry in fs::read_dir(&self.path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            fs::rename(entry.path(), destination.join(entry.file_name()))
                .map_err(|e| format!("Could not publish the result: {e}"))?;
        }
        let paths = relative
            .iter()
            .map(|path| destination.join(path).to_string_lossy().into_owned())
            .collect();
        published.keep = true;
        Ok(paths)
    }
}

fn validate_tree(path: &Path, depth: usize, count: &mut usize) -> Result<(), String> {
    use std::os::windows::fs::MetadataExt;
    *count += 1;
    if depth > 32 || *count > 4096 {
        return Err("The conversion produced too many output files.".into());
    }
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if metadata.file_attributes() & 0x400 != 0 {
        return Err("Linked output files are not accepted.".into());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            validate_tree(&entry.map_err(|e| e.to_string())?.path(), depth + 1, count)?;
        }
    }
    Ok(())
}

impl Drop for Staging {
    fn drop(&mut self) {
        // remove_dir_all on Windows does not follow symlinks or junctions.
        if !self.keep {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_publication_preserves_conflicts_and_rejects_escaped_outputs() {
        let root = std::env::temp_dir().join(format!("notch-test-{}", identity()));
        fs::create_dir(&root).unwrap();
        let owned = Staging {
            path: root.clone(),
            keep: false,
        };
        fs::write(root.join("photo.png"), b"original").unwrap();
        let stage = Staging::new(&root).unwrap();
        fs::write(stage.path.join("photo.jpg"), b"converted").unwrap();
        assert!(stage
            .publish(
                "photo.png",
                "jpg",
                &[root.join("photo.png").to_string_lossy().into()]
            )
            .is_err());
        fs::create_dir(root.join("photo-jpg")).unwrap();
        let outputs = stage
            .publish(
                "photo.png",
                "jpg",
                &[stage.path.join("photo.jpg").to_string_lossy().into()],
            )
            .unwrap();
        assert!(outputs[0].contains("photo-jpg (1)"));
        assert_eq!(fs::read(root.join("photo.png")).unwrap(), b"original");
        assert_eq!(fs::read(&outputs[0]).unwrap(), b"converted");
        drop(stage);
        drop(owned);
    }
}
