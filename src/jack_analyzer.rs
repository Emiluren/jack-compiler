use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashSet;

pub struct JackAnalyzer {
    data: Vec<char>,
    pos: usize,
    symbols: HashSet<char>,
}

pub enum TokenType {
    Keyword,
    Symbol,
    Identifier,
    IntConst,
    StringConst,
}

pub enum Keyword {
    Class,
    Method,
    Function,
    Constructor,
    Int,
    Boolean,
    Char,
    Void,
    Var,
    Static,
    Field,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
    True,
    False,
    Null,
    This,
}

impl JackAnalyzer {
    pub fn new(filename: &String) -> JackAnalyzer {
        let path = Path::new(filename);
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(why) => panic!("Couldn't open file {}: {}", display,
                               Error::description(&why)),
            Ok(file) => file,
        };

        let mut data_string = String::new();
        match file.read_to_string(&mut data_string) {
            Err(why) => panic!("couldn't read {}: {}", display,
                               Error::description(&why)),
            Ok(string) => string,
        };

        JackAnalyzer {
            data: data_string.chars().collect(),
            pos: 0,
            symbols: [
                '{', '}', '(', ')', '[', ']', '.',
                ',', ';', '+', '-', '*', '/', '&',
                '|', '<', '>', '=', '~',
            ].iter().cloned().collect(),
            keyword_map: keyword_map,
        }
    }

    pub fn has_more_tokens(&self) -> bool {
        let mut peek_pos = self.pos + 1;

        if self.data[self.pos].is_alphanumeric() {
            while self.data[peek_pos].is_alphanumeric() {
                peek_pos += 1;
                if peek_pos >= self.data.len() {
                    return false;
                }
            }
        }

        while self.data[peek_pos].is_whitespace() {
            peek_pos += 1;
            if peek_pos >= self.data.len() {
                return false;
            }
        }
        return true;
    }

    pub fn advance(&mut self) {
        let mut peek_pos = self.pos + 1;
        if self.data[self.pos].is_alphanumeric() {
            while self.data[peek_pos].is_alphanumeric() {
                peek_pos += 1;
            }
        }

        while self.data[peek_pos].is_whitespace() {
            peek_pos += 1;
        }
        self.pos = peek_pos;
    }

    pub fn token_type(&self) -> TokenType {
        let current_char = self.data[self.pos];

        if self.symbols.contains(&current_char) {
            return TokenType::Symbol;
        }

        if current_char == '\"' {
            return TokenType::StringConst;
        }

        if current_char.is_numeric() {
            return TokenType::IntConst;
        }

        if current_char.is_alphabetic() {
            let name = self.identifier();
            if self.keyword_map.contains_key(&*name) {
                return TokenType::Keyword;
            }
            return TokenType::Identifier;
        }

        panic!("YOLO!")
    }

    pub fn key_word(&self) -> Option<Keyword> {
        let name = self.identifier();
        return match name {
            "class" => Some(Keyword::Class),
            "method" => Some(Keyword::Method),
            "function" => Some(Keyword::Function),
            "constructor" => Some(Keyword::Constructor),
            "int" => Some(Keyword::Int),
            "boolean" => Some(Keyword::Boolean),
            "char" => Some(Keyword::Char),
            "void" => Some(Keyword::Void),
            "var" => Some(Keyword::Var),
            "static" => Some(Keyword::Static),
            "field" => Some(Keyword::Field),
            "let" => Some(Keyword::Let),
            "do" => Some(Keyword::Do),
            "if" => Some(Keyword::If),
            "else" => Some(Keyword::Else),
            "while" => Some(Keyword::While),
            "return" => Some(Keyword::Return),
            "true" => Some(Keyword::True),
            "false" => Some(Keyword::False),
            "null" => Some(Keyword::Null),
            "this" => Some(Keyword::This),
            _ => None,
        }
    }

    pub fn symbol(&self) -> char {
        self.data[self.pos]
    }

    pub fn identifier(&self) -> String {
        let mut buf = String::new();
        let mut peek_pos = self.pos;
        while self.data[peek_pos].is_alphanumeric() {
            buf.push(self.data[peek_pos]);
            peek_pos += 1;
        }
        return buf;
    }

    pub fn int_val(&self) -> i32 {
        let mut buf = String::new();
        let mut peek_pos = self.pos;
        while self.data[peek_pos].is_numeric() {
            buf.push(self.data[peek_pos]);
            peek_pos += 1;
        }
        return buf.parse::<i32>().unwrap();
    }

    pub fn string_val(&self) -> String {
        let mut peek_pos = self.pos + 1;
        let mut buf = String::new();

        while self.data[peek_pos] != '"' {
            buf.push(self.data[peek_pos]);
            peek_pos += 1;
        }

        return buf;
    }
}
