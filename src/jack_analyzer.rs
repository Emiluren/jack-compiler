use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashSet;

pub struct JackAnalyzer {
    data: Vec<char>,
    pos: usize,
    first_time: bool,
    symbols: HashSet<char>,
}

#[derive(PartialEq)]
pub enum TokenType {
    Keyword,
    Symbol,
    Identifier,
    IntConst,
    StringConst,
}

#[derive(PartialEq)]
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

// TODO: add function to print current line with line number
impl JackAnalyzer {
    pub fn new(path: &Path) -> JackAnalyzer {
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
            first_time: true,
            symbols: [
                '{', '}', '(', ')', '[', ']', '.',
                ',', ';', '+', '-', '*', '/', '&',
                '|', '<', '>', '=', '~',
            ].iter().cloned().collect(),
        }
    }

    fn start_of_token(&self, pos: usize) -> bool {
        if pos >= self.data.len() {
            return false;
        }

        if self.data[pos].is_whitespace() {
            return false;
        }

        if self.data[pos] == '/' {
            if pos + 1 >= self.data.len() {
                return true;
            }

            let next_char = self.data[pos+1];
            if next_char == '/' || next_char == '*' {
                return false;
            }
        }
        return true;
    }

    fn skip_comments_and_whitespace(&self, start_pos: usize) -> Option<usize> {
        let mut peek_pos = start_pos;
        while peek_pos < self.data.len() {
            // Skip comments
            if self.data[peek_pos] == '/' {
                if peek_pos + 1 >= self.data.len() {
                    return Some(peek_pos);
                }

                // Skip multiline comments
                if self.data[peek_pos+1] == '/' {
                    while self.data[peek_pos] != '\n' {
                        peek_pos += 1;
                        if peek_pos >= self.data.len() {
                            return None;
                        }
                    }
                } else if self.data[peek_pos+1] == '*' {
                    while !(self.data[peek_pos] == '*' && self.data[peek_pos+1] == '/') {
                        peek_pos += 1;
                        if peek_pos + 1 >= self.data.len() {
                            return None;
                        }
                    }
                    peek_pos += 2;
                }
            }

            // Skip whitespace
            while self.data[peek_pos].is_whitespace() {
                peek_pos += 1;
                if peek_pos >= self.data.len() {
                    return None;
                }
            }

            if self.start_of_token(peek_pos) {
                return Some(peek_pos);
            }
        }
        return None;
    }

    fn pos_of_next_token(&self) -> Option<usize> {
        let mut peek_pos = self.pos;
        
        if !self.first_time {
            peek_pos += 1;
        }

        if self.data[self.pos] == '\"' {
            while self.data[peek_pos] != '\"' {
                peek_pos += 1;
            }
            peek_pos += 1;
        } else if peek_pos > 0 && self.data[self.pos].is_alphanumeric() {
            while self.data[peek_pos].is_alphanumeric() {
                peek_pos += 1;
                if peek_pos >= self.data.len() {
                    return None;
                }
            }
        }

        let skip_result = self.skip_comments_and_whitespace(peek_pos);
        return skip_result;
    }

    #[allow(dead_code)]
    pub fn has_more_tokens(&self) -> bool {
        match self.pos_of_next_token() {
            Some(_) => true,
            None => false,
        }
    }

    pub fn advance(&mut self) {
        self.pos = self.pos_of_next_token().unwrap();
        self.first_time = false;
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
            return match self.key_word() {
                Some(_) => TokenType::Keyword,
                None => TokenType::Identifier,
            }
        }

        panic!("{} was nothing", current_char)
    }

    pub fn key_word(&self) -> Option<Keyword> {
        let name = self.identifier();
        return match &*name {
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
