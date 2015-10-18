use jack_analyzer::*;
use xml_output::*;

use std::fs::File;
use std::io::prelude::*;

pub struct CompilationEngine {
    analyzer: JackAnalyzer,
    outfile: File,
}

impl CompilationEngine {
    pub fn new(infile: &str, outfile: &str) -> CompilationEngine {
        CompilationEngine {
            analyzer: JackAnalyzer::new(infile),
            outfile: File::create(outfile).unwrap(),
        }
    }

    pub fn compile_class(&mut self) {
        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Keyword || self.analyzer.key_word().unwrap() != Keyword::Class {
            panic!("File must start with class");
        }

        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Identifier {
            panic!("No class name");
        }

        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '{' {
            panic!("Missing opening brace");
        }

        self.outfile.write_all(b"<class>\n").unwrap();
        while self.analyzer.has_more_tokens() {
            self.analyzer.advance();
        }
        self.outfile.write_all(b"</class>\n").unwrap();
    }

    pub fn compile_class_var_dec() {
    }

    pub fn compile_subroutine() {
    }

    pub fn compile_parameter() {
    }

    pub fn compile_var_dec() {
    }

    pub fn compile_statements() {
    }

    pub fn compile_do() {
    }

    pub fn compile_let() {
    }

    pub fn compile_while() {
    }

    pub fn compile_return() {
    }

    pub fn compile_if() {
    }

    pub fn compile_expression() {
    }

    pub fn compile_term() {
    }

    pub fn compile_expression_list() {
    }
}
