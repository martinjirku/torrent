use std::{collections::HashMap, fmt::{self, Debug}};

use super::bencode::Bencode;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TorrentFile {
    pub announce: String,
    pub created_by: Option<String>,
    pub creation_date: Option<i64>,
    pub info: Info,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Info {
    pub files: Option<Vec<File>>,
    pub length: Option<i64>,
    pub name: String,
    pub piece_length: i64,
    pub pieces: Pieces,
}

pub struct Pieces(pub Vec<[u8; 20]>);

impl Debug for Pieces {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text: Vec<String> = self.0
            .iter()
            .map(|piece| format!("\"{}\"", &percent_encode(&piece)))
            .collect();
        write!(f, "Pieces {{ data: [{}] }}", text.join(", "))
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct File {
    pub length: i64,
    pub path: Vec<String>,
}

impl TorrentFile {
    pub fn from_bencode(data: &Bencode) -> Result<TorrentFile, String> {
        match data {
            Bencode::Dict(data, _) => Ok(TorrentFile {
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
            Bencode::Dict(data, _) => Ok(Info {
                files: match data.get("files") {
                    Some(Bencode::List(b_files, _)) => {
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
                pieces: extract_pieces(data)?,
            }),
            _ => Err(String::from("Expected dictionary for info")),
        }
    }
}

impl File {
    fn from_bencode(data: &Bencode) -> Result<File, String> {
        match data {
            Bencode::Dict(data, _) => Ok(File {
                length: extract_i64(data, "length")?,
                path: match data.get("path") {
                    Some(Bencode::List(p, _)) => {
                        let mut paths = vec![];
                        for p in p {
                            match p {
                                Bencode::String(s, _) => match String::from_utf8(s.clone()) {
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
        Some(Bencode::String(s, _)) => match String::from_utf8(s.clone()) {
            Ok(s) => Ok(s),
            Err(_) => return Err(String::from("Invalid announce string")),
        },
        _ => return Err(String::from("Invalid announce string")),
    }
}
fn extract_optional_string(data: &HashMap<String, Bencode>, key: &str) -> Result<Option<String>, String> {
    match data.get(key) {
        Some(created_by) => match created_by {
            Bencode::String(s, _) => match String::from_utf8(s.clone()) {
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
        Some(Bencode::Int(i,_)) => Ok(i.clone()),
        _ => return Err(String::from("Invalid i64")),
    }
}

fn extract_option_i64(data: &HashMap<String, Bencode>, key: &str) -> Result<Option<i64>, String> {
    match data.get(key) {
        Some(Bencode::Int(i,_)) => Ok(Some(i.clone())),
        Some(_) => return Err(String::from("Invalid option type")),
        None => Ok(None),
    }
}

pub fn percent_encode(bytes: &[u8; 20]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 3);
    for &byte in bytes {
        encoded.push('%');
        encoded.push_str(&format!("{:02X}", byte));
    }
    encoded
}

fn extract_pieces(data: &HashMap<String, Bencode>) -> Result<Pieces, String> {
    let mut pieces = vec![];
    match data.get("pieces") {
        Some(Bencode::String(s, _)) => {
            let mut i = 0;
            while i < s.len() {
                let piece: [u8; 20] = s[i..i+20].try_into().map_err(|_| "Invalid piece length")?;
                pieces.push(piece);
                i += 20;
            }
            Ok(Pieces(pieces) )
        },
        Some(_) => return Err(String::from("Invalid pieces type")),
        None => Ok(Pieces(pieces)),
    }   
}

fn _sha1_from_torrent_file(_data: &str) -> &[u8; 20] {
    todo!()
}