use std::collections::HashMap;
use std::io::Read;

#[derive(Debug, PartialEq)]
pub enum Bencode {
    Int(i64),
    String(String),
    List(Vec<Bencode>),
    Dict(HashMap<String, Bencode>),
}

enum Token {
    None,
    Int(i64),
    String(String),
    List,
    DictStart,
    DictEnd,
}

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


struct Tokenizer {
    data: String,
    index: usize,
}

impl Tokenizer {
    fn new<'a>(data: String) -> Tokenizer {
        Tokenizer{
            data,
            index: 0,
        }
    }
    fn next(&mut self) -> Result<Token, String> {
        if self.index >= self.data.len() {
            return Err("No more tokens".to_string());
        }
        let c = self.data.chars().nth(self.index);
        match c {
            // Int := "i" IntValue "e"
            Some('i') => self.next_int(),
            Some(c) if c.is_digit(10) => self.next_string(),
            _ => Err("Invalid token".to_string()),
        }
    }
    fn next_string(&mut self) -> Result<Token, String> {
        let start = self.index;
        while let Some(c) = self.data.chars().nth(self.index) {
            match c {
                '0'..='9' => self.index += 1,
                ':' => break,
                _ => return Err("Invalid token in string".to_string()),
            }
        }

        let length: usize = self.data[start..self.index]
            .parse::<usize>()
            .map_err(|e| e.to_string())?;
        let string = self.data[self.index+1..self.index+length+1].to_string();
        self.index += length + 1;
        Ok(Token::String(string))
    }
    fn next_int(&mut self) -> Result<Token, String> {
        self.index += 1; // skip 'i'
        let start = self.index;
        while let Some(c) = self.data.chars().nth(self.index) {
            if c == 'e' {
                let value = self
                    .data[start..self.index]
                    .parse()
                    .map_err(|_| "invalid token".to_string())?;
                self.index += 1; // skip 'e'
                return Ok(Token::Int(value));
            }
            self.index += 1;
        }
        Err("Invalid token".to_string())
    }
}

pub struct Parser {
    tokenizer: Tokenizer,
    next_token: Result<Token, String>,
}

impl Parser {
    pub fn new<T: Read>(reader: &mut T) -> Parser {
        let mut data: String = String::new();
        reader.read_to_string(&mut data).unwrap();
        Parser{
            tokenizer: Tokenizer::new(data),
            next_token: Ok(Token::None),
        }
    }
    pub fn parse(&mut self) -> Result<Bencode, String> {
        self.next_token = self.tokenizer.next();
        match &self.next_token {
            Ok(Token::Int(value)) => Ok(Bencode::Int(*value)),
            Ok(Token::String(value)) => Ok(Bencode::String(value.clone())),
            Ok(Token::DictStart) => self.parse_dict(),
            Err(e) => Err(e.clone()),
            _ => Err("Unexpected token".to_string()),
        }
    }

    fn parse_dict(&mut self) -> Result<Bencode, String> {
        // let mut dict = HashMap::new();
        todo!();
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
        assert_eq!(bencode, Ok(Bencode::String("spam".to_string())));
    }

    // #[test]
    // fn test_decode_normal() {
    //     let mut reader = std::io::Cursor::new("d3:cow3:moo4:spam4:eggse");
    //     let parser = BencodeParser::new(&mut reader);
    //     let bencode = parser.parse();
    //     assert_eq!(bencode, Bencode::Dict(
    //         vec![
    //             ("cow".to_string(), Bencode::String("moo".to_string())),
    //             ("spam".to_string(), Bencode::String("eggs".to_string()))
    //         ].into_iter().collect()
    //     ));
    // }
}
