mod jack_analyzer;
use jack_analyzer::*;
use std::env;
use std::fs::File;

fn write_tag(tag_name: &str, content: &str) -> String {
    format!("<{0}> {1} </{0}>\n", tag_name, content)
}

fn keyword_to_str(keyword: Keyword) -> &'static str {
    match keyword {
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

fn get_tag_name(token_type: TokenType) -> &'static str {
    match token_type {
        TokenType::Keyword => "keyword",
        TokenType::Symbol => "symbol",
        TokenType::Identifier => "identifier",
        TokenType::IntConst => "integerConstant",
        TokenType::StringConst => "stringConstant",
    }
}

fn get_tag_data(analyzer: &JackAnalyzer, token_type: TokenType) -> String {
    match token_type {
        TokenType::Keyword => keyword_to_str(analyzer.key_word()).to_string(),
        TokenType::Symbol => analyzer.symbol().to_string(),
        TokenType::Identifier => analyzer.identifier(),
        TokenType::IntConst => analyzer.int_val().to_string(),
        TokenType::StringConst => analyzer.string_val(),
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 1 {
        println!("usage: jackcompiler filename");
    } else {
        let mut analyzer = JackAnalyzer::new(&args[1]);
        let mut outfile = File::create("output.xml").unwrap();

        while analyzer.has_more_tokens() {
            analyzer.advance();
            let token_type = analyzer.token_type();
            let tag_data = get_tag_data(&analyzer, token_type);
        }
    }
}
