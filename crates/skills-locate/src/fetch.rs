use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;

use crate::{Error, GitHubRef, Result};

pub type FileBuffer = HashMap<PathBuf, Vec<u8>>;

pub fn fetch_single_file(url: &str) -> Result<Vec<u8>> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| Error::Http(e.to_string()))?;

    let bytes = response
        .into_body()
        .read_to_vec()
        .map_err(|e| Error::Http(e.to_string()))?;

    Ok(bytes)
}

pub fn fetch_github_archive(github_ref: &GitHubRef) -> Result<FileBuffer> {
    let archive_url = github_ref.archive_url();

    let response = ureq::get(&archive_url)
        .call()
        .map_err(|e| Error::Http(e.to_string()))?;

    let bytes = response
        .into_body()
        .read_to_vec()
        .map_err(|e| Error::Http(e.to_string()))?;

    extract_zip_to_buffer(&bytes, github_ref.path.as_deref())
}

pub fn extract_zip_to_buffer(zip_bytes: &[u8], subpath: Option<&str>) -> Result<FileBuffer> {
    let reader = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| Error::Zip(e.to_string()))?;

    let mut files = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| Error::Zip(e.to_string()))?;

        if file.is_dir() {
            continue;
        }

        let raw_path = file.name().to_string();

        let path_parts: Vec<&str> = raw_path.splitn(2, '/').collect();
        if path_parts.len() < 2 {
            continue;
        }
        let relative_path = path_parts[1];

        if relative_path.is_empty() {
            continue;
        }

        if let Some(sub) = subpath {
            if !relative_path.starts_with(sub) {
                continue;
            }
            let trimmed = relative_path.strip_prefix(sub).unwrap_or(relative_path);
            let trimmed = trimmed.trim_start_matches('/');
            if trimmed.is_empty() {
                continue;
            }

            let mut contents = Vec::new();
            file.read_to_end(&mut contents).map_err(Error::Io)?;
            files.insert(PathBuf::from(trimmed), contents);
        } else {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).map_err(Error::Io)?;
            files.insert(PathBuf::from(relative_path), contents);
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_extract_zip_strips_root_directory() {
        let mut buffer = Vec::new();
        {
            let writer = std::io::Cursor::new(&mut buffer);
            let mut zip = zip::ZipWriter::new(writer);

            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("repo-main/file.txt", options).unwrap();
            zip.write_all(b"content").unwrap();
            zip.start_file("repo-main/subdir/other.txt", options)
                .unwrap();
            zip.write_all(b"other").unwrap();
            zip.finish().unwrap();
        }

        let files = extract_zip_to_buffer(&buffer, None).unwrap();

        assert!(files.contains_key(&PathBuf::from("file.txt")));
        assert!(files.contains_key(&PathBuf::from("subdir/other.txt")));
        assert_eq!(files.get(&PathBuf::from("file.txt")).unwrap(), b"content");
    }

    #[test]
    fn test_extract_zip_with_subpath_filter() {
        let mut buffer = Vec::new();
        {
            let writer = std::io::Cursor::new(&mut buffer);
            let mut zip = zip::ZipWriter::new(writer);

            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("repo-main/skills/auth/skill.md", options)
                .unwrap();
            zip.write_all(b"auth skill").unwrap();
            zip.start_file("repo-main/skills/db/skill.md", options)
                .unwrap();
            zip.write_all(b"db skill").unwrap();
            zip.start_file("repo-main/README.md", options).unwrap();
            zip.write_all(b"readme").unwrap();
            zip.finish().unwrap();
        }

        let files = extract_zip_to_buffer(&buffer, Some("skills/auth")).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files.contains_key(&PathBuf::from("skill.md")));
    }
}
