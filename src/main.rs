mod jack_analyzer;
mod compilation_engine;
use jack_analyzer::*;
use compilation_engine::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn write_tag(tag_name: &str, content: &str) -> String {
    format!("<{0}> {1} </{0}>\n", tag_name, content)
}

fn keyword_to_str(keyword: &Keyword) -> &'static str {
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

fn get_tag_name(token_type: &TokenType) -> &'static str {
    match *token_type {
        TokenType::Keyword => "keyword",
        TokenType::Symbol => "symbol",
        TokenType::Identifier => "identifier",
        TokenType::IntConst => "integerConstant",
        TokenType::StringConst => "stringConstant",
    }
}

fn get_tag_data(analyzer: &JackAnalyzer, token_type: &TokenType) -> String {
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

fn write_token_file(infile: &String) {
    let localname = infile.split('/').last().unwrap();
    let outname = localname.split('.').next().unwrap().to_string() + ".xml";
    println!("Compiling {} to {}", infile, outname);
    let mut analyzer = JackAnalyzer::new(infile);
    let mut outfile = File::create(outname).unwrap();
    outfile.write_all(b"<tokens>\n").unwrap();

    while analyzer.has_more_tokens() {
        analyzer.advance();
        let token_type = analyzer.token_type();
        let tag_data = get_tag_data(&analyzer, &token_type);
        outfile.write_all(write_tag(get_tag_name(&token_type), &tag_data).as_bytes()).unwrap();
    }
    outfile.write_all(b"</tokens>\n").unwrap();
    println!("YOLO!");
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut current_file: usize = 1;
    if args.len() < 2 {
        println!("usage: jackcompiler filename");
    } else {
        while current_file < args.len() {
            let filename = &args[current_file];
            write_token_file(filename);
            current_file += 1;
        }
    }
}
