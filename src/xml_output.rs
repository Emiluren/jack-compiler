use jack_analyzer::*;

use std::fs::File;
use std::io::prelude::*;

pub fn tag_string(tag_name: &str, content: &str) -> String {
    format!("<{0}> {1} </{0}>\n", tag_name, content)
}

pub fn keyword_to_str(keyword: &Keyword) -> &'static str {
    match *keyword {
        Keyword::Class => "class",
        Keyword::Method => "method",
        Keyword::Function => "function",
        Keyword::Constructor => "constructor",
        Keyword::Int => "int",
        Keyword::Boolean => "boolean",
        Keyword::Char => "char",
        Keyword::Void => "void",
        Keyword::Var => "var",
        Keyword::Static => "static",
        Keyword::Field => "field",
        Keyword::Let => "let",
        Keyword::Do => "do",
        Keyword::If => "if",
        Keyword::Else => "else",
        Keyword::While => "while",
        Keyword::Return => "return",
        Keyword::True => "true",
        Keyword::False => "false",
        Keyword::Null => "null",
        Keyword::This => "this",
    }
}

pub fn get_tag_name(token_type: &TokenType) -> &'static str {
    match *token_type {
        TokenType::Keyword => "keyword",
        TokenType::Symbol => "symbol",
        TokenType::Identifier => "identifier",
        TokenType::IntConst => "integerConstant",
        TokenType::StringConst => "stringConstant",
    }
}

pub fn get_tag_data(analyzer: &JackAnalyzer, token_type: &TokenType) -> String {
    match *token_type {
        TokenType::Keyword => keyword_to_str(&analyzer.key_word().unwrap()).to_string(),
        TokenType::Symbol => match analyzer.symbol() {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            other => other.to_string(),
        },
        TokenType::Identifier => analyzer.identifier(),
        TokenType::IntConst => analyzer.int_val().to_string(),
        TokenType::StringConst => analyzer.string_val(),
    }
}

pub fn make_tag_string(analyzer: &JackAnalyzer) -> String {
    let token_type = analyzer.token_type();
    let tag_data = get_tag_data(&analyzer, &token_type);
    tag_string(get_tag_name(&token_type), &tag_data)
}

pub fn write_tag_string(analyzer: &JackAnalyzer, outfile: &mut File) {
    outfile.write_all(make_tag_string(&analyzer).as_bytes()).unwrap();
}
