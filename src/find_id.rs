use glob::glob;
use rayon::prelude::*;
use roxmltree::Document;
use std::{fs, path::PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub enum SearchMode {
    MediaID,
    Guid,
    ShortID,
}


#[derive(Clone, Debug)]
pub struct MatchInfo {
    pub tag: String,
    pub name: String,
    pub id: String,
    pub short_id: String,
    pub media_id: String,
    pub language: String,
    pub audio_file: String,
}

pub fn is_path_valid(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);

    if !path.is_dir() {
        return Err(format!(
            "Path '{}' is not a valid directory",
            path.display()
        ));
    }

    let has_wproj = path
        .read_dir()
        .map_err(|e| format!("Failed to read directory '{}': {}", path.display(), e))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .any(|p| p.is_file() && p.extension().map_or(false, |ext| ext == "wproj"));

    if has_wproj {
        Ok(path)
    } else {
        Err(format!(
            "No '.wproj' files found in directory '{}'",
            path.display()
        ))
    }
}

pub fn find_id(query: &str, path: &str, mode: &SearchMode) -> Vec<MatchInfo> {
    let query = query.to_lowercase();
    let pattern = format!("{}/**/*.wwu", path);
    let entries: Vec<PathBuf> = glob(&pattern)
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect();

    let results: Vec<MatchInfo> = entries
        .par_iter()
        .flat_map_iter(|p| {
            let contents = match fs::read_to_string(p) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("读取文件失败：{} {}", p.display(), e);
                    return Vec::new();
                }
            };
            match mode {
                SearchMode::MediaID => search_media_id(&query, &contents),
                SearchMode::Guid => search_guid(&query, &contents),
                SearchMode::ShortID => search_short_id(&query, &contents),
            }
        })
        .collect();

    results
}

fn search_media_id(query: &str, contents: &str) -> Vec<MatchInfo> {
    let doc = Document::parse(contents).unwrap();
    let mut results = Vec::new();

    for node in doc.descendants().filter(|n| n.has_tag_name("MediaID")) {
        let id = node.attribute("ID").unwrap_or("?");
        if id.to_lowercase().contains(query) {
            let parent = node.parent_element().unwrap().parent_element().unwrap();

            results.push(MatchInfo {
                tag: parent.tag_name().name().to_string(),
                name: parent.attribute("Name").unwrap_or("?").to_string(),
                id: parent.attribute("ID").unwrap_or("?").to_string(),
                short_id: "?".to_string(),
                media_id: id.to_string(),
                language: parent
                    .children()
                    .find(|n| n.tag_name().name().contains("Language"))
                    .and_then(|n| n.text())
                    .unwrap_or("?")
                    .to_string(),
                audio_file: parent
                    .children()
                    .find(|n| n.tag_name().name().contains("AudioFile"))
                    .and_then(|n| n.text())
                    .unwrap_or("?")
                    .to_string(),
            });
        }
    }
    results
}

fn search_guid(query: &str, contents: &str) -> Vec<MatchInfo> {
    let doc = Document::parse(contents).unwrap();
    let mut results = Vec::new();

    for node in doc.descendants().filter(|n| n.has_attribute("ID")) {
        let id = node.attribute("ID").unwrap_or("?");
        if id.to_lowercase().contains(query) {
            results.push(MatchInfo {
                tag: node.tag_name().name().to_string(),
                name: node.attribute("Name").unwrap_or("?").to_string(),
                id: id.to_string(),
                short_id: node.attribute("ShortID").unwrap_or("?").to_string(),
                media_id: "".to_string(),
                language: "".to_string(),
                audio_file: "".to_string(),
            });
        }
    }
    results
}

fn search_short_id(query: &str, contents: &str) -> Vec<MatchInfo> {
    let doc = Document::parse(contents).unwrap();
    let mut results = Vec::new();

    for node in doc.descendants().filter(|n| n.has_attribute("ShortID")) {
        let short_id = node.attribute("ShortID").unwrap_or("?");
        if short_id.to_lowercase().contains(query) {
            results.push(MatchInfo {
                tag: node.tag_name().name().to_string(),
                name: node.attribute("Name").unwrap_or("?").to_string(),
                id: node.attribute("ID").unwrap_or("?").to_string(),
                short_id: short_id.to_string(),
                media_id: "".to_string(),
                language: "".to_string(),
                audio_file: "".to_string(),
            });
        }
    }
    results
}
