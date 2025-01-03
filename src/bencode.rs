use std::collections::HashMap;
use std::io::Read;

/// Bencode is a simple encoding format used by BitTorrent clients.
/// It is used to encode dictionaries, lists, integers, and strings.
/// 
/// Bencode :=
///   Int
///   | String
///   | List
///   | Dict
/// 
/// Int := "i" IntValue "e"
/// 
/// IntValue := "-" [1-9] [0-9]*
///          | "0"
/// 
/// String := StringLength ":" StringValue
/// StringLength := [1-9] [0-9]*
/// StringValue := [\x20-\x7E\x80-\xFF]* 
/// 
/// List := "l" Bencode* "e"
/// 
/// Dict := "d" DictEntry* "e"
/// DictEntry := String Bencode
/// 
/// Example:
/// 
/// "d3:cow3:moo4:spam4:eggse" -> Dict {
///    "cow" => String("moo"),
///   "spam" => String("eggs"),
/// }

#[derive(Debug, PartialEq)]
pub enum Bencode {
    Int(i64),
    String(Vec<u8>),
    List(Vec<Bencode>),
    Dict(HashMap<String, Bencode>),
}

enum Token {
    Int(i64),
    String(Vec<u8>),
    ListStart,
    DictStart,
    ListDictEnd,
}

struct Tokenizer {
    data: Vec<u8>,
    index: usize,
}

impl Tokenizer {
    fn new<'a>(data: Vec<u8>) -> Tokenizer {
        Tokenizer{
            data,
            index: 0,
        }
    }
    fn next(&mut self) -> Result<Token, String> {
        if self.index >= self.data.len() {
            return Err("No more tokens".to_string());
        }
        let c = self.data[self.index] as char;
        match c {
            // Int := "i" IntValue "e"
            'i' => self.next_int(),
            '0'..='9' => self.next_string(),
            'd' => {
                self.index += 1;
                Ok(Token::DictStart)
            },
            'e' => {
                self.index += 1;
                Ok(Token::ListDictEnd)
            },
            'l' => {
                self.index += 1;
                Ok(Token::ListStart)
            },
            _ => Err("Invalid token".to_string()),
        }
    }
    fn next_string(&mut self) -> Result<Token, String> {
        let start = self.index;
        loop {
            let c = self.data[self.index];
            match c as char {
                '0'..='9' => self.index += 1,
                ':' => break,
                _ => return Err("Invalid token in string".to_string()),
            }
        }

        let length: usize = std::str::from_utf8(&self.data[start..self.index])
            .map_err(|e| e.to_string())?
            .parse::<usize>()
            .map_err(|e| e.to_string())?;
        let string = self.data[self.index+1..self.index+length+1].to_vec();
        self.index += length + 1;
        Ok(Token::String(string))
    }
    fn next_int(&mut self) -> Result<Token, String> {
        self.index += 1; // skip 'i'
        let start = self.index;
        loop {
            let c = self.data[self.index] as char;
            if c == 'e' {
                let value = String::from_utf8(self.data[start..self.index].to_vec())
                    .map_err(|e| e.to_string())?
                    .parse::<i64>()
                    .map_err(|e| e.to_string())?;
                self.index += 1; // skip 'e'
                return Ok(Token::Int(value));
            }
            self.index += 1;
        }
    }
}

pub struct Parser {
    tokenizer: Tokenizer,
}

impl Parser {
    pub fn new<T: Read>(reader: &mut T) -> Parser {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        Parser{
            tokenizer: Tokenizer::new(buffer),
        }
    }
    /// Parse the bencode data and return a Bencode enum
    /// This implementation is using a recursive descent parser algorithm
    pub fn parse(&mut self) -> Result<Bencode, String> {
        let next_token = self.tokenizer.next();
        match next_token {
            Ok(Token::Int(value)) => Ok(Bencode::Int(value)),
            Ok(Token::String(value)) => Ok(Bencode::String(value.clone())),
            Ok(Token::DictStart) => self.parse_dict(),
            Ok(Token::ListStart) => self.parse_list(),
            Err(e) => Err(format!("parse: {}", e)),
            _ => Err("Unexpected token".to_string()),
        }
    }

    fn parse_dict(&mut self) -> Result<Bencode, String> {
        let mut dict = HashMap::new();
        loop {
            let dict_key = match self.tokenizer.next() {
                Ok(Token::String(key)) => match String::from_utf8(key) {
                    Ok(value) => value,
                    Err(e) => return Err(e.to_string()),
                },
                Ok(Token::ListDictEnd) => return Ok(Bencode::Dict(dict)),
                Err(e) => return Err(e),
                _ => return Err("Unexpected token".to_string()),
            };
            let value_token = match self.tokenizer.next() {
                Ok(Token::Int(value)) => Bencode::Int(value),
                Ok(Token::String(value)) => Bencode::String(value.clone()),
                Ok(Token::DictStart) => match self.parse_dict() {
                    Ok(value) => value,
                    Err(e) => return Err(e),
                },
                Ok(Token::ListStart) => match self.parse_list() {
                    Ok(value) => value,
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(format!("parse_dict > parsing value for '{}' key: {}", dict_key, e)),
                _ => return Err("Unexpected token".to_string()),
            };
            dict.insert(dict_key, value_token);
        }
    }
    fn parse_list(&mut self) -> Result<Bencode, String> {
        let mut list = Vec::new();
        loop {
            let token = match self.tokenizer.next() {
                Ok(Token::Int(value)) => Bencode::Int(value),
                Ok(Token::String(value)) => Bencode::String(value.clone()),
                Ok(Token::DictStart) => match self.parse_dict() {
                    Ok(value) => value,
                    Err(e) => return Err(e.clone()),
                },
                Ok(Token::ListStart) => match self.parse_list() {
                    Ok(value) => value,
                    Err(e) => return Err(e.clone()),
                },
                Ok(Token::ListDictEnd) => return Ok(Bencode::List(list)),
                Err(e) => return Err(e.clone()),
            };
            list.push(token);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_int_42() {
        let mut reader = std::io::Cursor::new("i42e");
        let mut parser = Parser::new(&mut reader);
        let bencode = parser.parse();
        assert_eq!(bencode, Ok(Bencode::Int(42)));
    }
    #[test]
    fn test_decode_int_minus_42() {
        let mut reader = std::io::Cursor::new("i-42e");
        let mut parser = Parser::new(&mut reader);
        let bencode = parser.parse();
        assert_eq!(bencode, Ok(Bencode::Int(-42)));
    }

    #[test]
    fn test_decode_string() {
        let mut reader = std::io::Cursor::new("4:spam");
        let mut parser = Parser::new(&mut reader);
        let bencode = parser.parse();
        assert_eq!(bencode, Ok(Bencode::String(b"spam".to_vec())));
    }

    #[test]
    fn test_decode_dict() {
        let mut reader = std::io::Cursor::new("d3:cow3:moo4:spam4:eggse");
        let mut parser = Parser::new(&mut reader);
        let bencode = parser.parse();
        assert_eq!(bencode, Ok(Bencode::Dict(
            vec![
                ("cow".to_string(), Bencode::String(b"moo".to_vec())),
                ("spam".to_string(), Bencode::String(b"eggs".to_vec()))
            ].into_iter().collect()
        )));
    }

    #[test]
    fn test_decode_list() {
        let mut reader = std::io::Cursor::new("l4:spam4:eggse");
        let mut parser = Parser::new(&mut reader);
        let bencode = parser.parse();
        assert_eq!(bencode, Ok(Bencode::List(
            vec![
                Bencode::String(b"spam".to_vec()),
                Bencode::String(b"eggs".to_vec())
            ]
        )));
    }
}
