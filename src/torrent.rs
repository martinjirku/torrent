use std::collections::HashMap;

use super::bencode::Bencode;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TorrentFile {
    announce: String,
    created_by: Option<String>,
    creation_date: Option<i64>,
    info: Info,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Info {
    files: Option<Vec<File>>,
    length: Option<i64>,
    name: String,
    piece_length: i64,
    pieces: Vec<u8>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct File {
    length: i64,
    path: Vec<String>,
}

impl TorrentFile {
    pub fn from_bencode(data: &Bencode) -> Result<TorrentFile, String> {
        match data {
            Bencode::Dict(data) => Ok(TorrentFile {
                announce: extract_string(data, "announce")?,
                creation_date: extract_option_i64(data, "creation date")?,
                created_by: extract_optional_string(data, "created by")?,
                info: match data.get("info") {
                    Some(info) => match Info::from_bencode(info) {
                        Ok(info) => info,
                        Err(e) => return Err(e),
                    },
                    None => return Err(String::from("Missing info")),
                },
            }),
            _ => return Err(String::from("Expected dictionary")),
        }
        
    }
}
impl Info {
    fn from_bencode(data: &Bencode) -> Result<Info, String> {
        match data {
            Bencode::Dict(data) => Ok(Info {
                files: match data.get("files") {
                    Some(Bencode::List(b_files)) => {
                        let mut files = vec![];
                        for file in b_files {
                            match File::from_bencode(file) {
                                Ok(file) => files.push(file),
                                Err(_) => return Err(String::from("Invalid file")),
                            }
                        }
                        Some(files)
                    },
                    Some(_) => return Err(String::from("Invalid files type")),
                    None => None,
                },
                length: extract_option_i64(data, "length")?,
                name: extract_string(data, "name")?,
                piece_length: extract_i64(data, "piece length")?,
                pieces: match data.get("pieces") {
                    Some(Bencode::String(s)) => s.clone(),
                    _ => return Err(String::from("Invalid pieces")),
                },
            }),
            _ => Err(String::from("Expected dictionary for info")),
        }
    }
}

impl File {
    fn from_bencode(data: &Bencode) -> Result<File, String> {
        match data {
            Bencode::Dict(data) => Ok(File {
                length: extract_i64(data, "length")?,
                path: match data.get("path") {
                    Some(Bencode::List(p)) => {
                        let mut paths = vec![];
                        for p in p {
                            match p {
                                Bencode::String(s) => match String::from_utf8(s.clone()) {
                                    Ok(s) => paths.push(s),
                                    Err(_) => return Err(String::from("Invalid path string")),
                                },
                                _ => return Err(String::from("Invalid path type")),
                            }
                        }
                        paths
                    },
                    _ => return Err(String::from("Invalid path, expected list")),
                },
            }),
            _ => Err(String::from("Expected dictionary for file")),
        }
    }
}

// helper functions

fn extract_string(data: &HashMap<String, Bencode>, key: &str) -> Result<String, String> {
    match data.get(key) {
        Some(Bencode::String(s)) => match String::from_utf8(s.clone()) {
            Ok(s) => Ok(s),
            Err(_) => return Err(String::from("Invalid announce string")),
        },
        _ => return Err(String::from("Invalid announce string")),
    }
}
fn extract_optional_string(data: &HashMap<String, Bencode>, key: &str) -> Result<Option<String>, String> {
    match data.get(key) {
        Some(created_by) => match created_by {
            Bencode::String(s) => match String::from_utf8(s.clone()) {
                Ok(s) => Ok(Some(s.clone())),
                _ => return Err(String::from("Invalid string")),
            },
            _ => return Err(String::from("Invalid created by type")),
        },
        None => Ok(None),
    }
}
fn extract_i64(data: &HashMap<String, Bencode>, key: &str) -> Result<i64, String> {
    match data.get(key) {
        Some(Bencode::Int(i)) => Ok(i.clone()),
        _ => return Err(String::from("Invalid i64")),
    }
}

fn extract_option_i64(data: &HashMap<String, Bencode>, key: &str) -> Result<Option<i64>, String> {
    match data.get(key) {
        Some(Bencode::Int(i)) => Ok(Some(i.clone())),
        Some(_) => return Err(String::from("Invalid option type")),
        None => Ok(None),
    }
}