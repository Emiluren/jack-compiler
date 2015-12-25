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
        self.outfile.write_all(b"<class>\n").unwrap();
        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Keyword || self.analyzer.key_word().unwrap() != Keyword::Class {
            panic!("File must start with class");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Identifier {
            panic!("No class name");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '{' {
            panic!("Missing opening brace");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        let is_class_var = |keyword: Keyword| keyword == Keyword::Static ||
            keyword == Keyword::Field;

        let is_subroutine = |keyword: Keyword| match keyword {
            Keyword::Constructor | Keyword::Function | Keyword::Method => true,
            _ => false
        };

        self.analyzer.advance();
        while !(self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == '}') {
            match self.analyzer.token_type() {
                TokenType::Keyword if is_class_var(self.analyzer.key_word().unwrap()) => self.compile_class_var_dec(),
                TokenType::Keyword if is_subroutine(self.analyzer.key_word().unwrap()) => self.compile_subroutine(),
                _ => panic!("Unknown token inside class: ".to_string() +
                            &make_tag_string(&self.analyzer)),
            };
        }
        write_tag_string(&self.analyzer, &mut self.outfile);
        self.outfile.write_all(b"</class>\n").unwrap();
    }

    pub fn compile_class_var_dec(&mut self) {
        self.outfile.write_all(b"<classVarDec>\n").unwrap();
        fn end_of_class_var(analyzer: &JackAnalyzer) -> bool {
            if analyzer.token_type() != TokenType::Symbol {
                return false;
            }
            analyzer.symbol() == ';'
        }

        while !end_of_class_var(&self.analyzer) {
            write_tag_string(&self.analyzer, &mut self.outfile);
            self.analyzer.advance();
        }
        write_tag_string(&self.analyzer, &mut self.outfile); // Write semicolon
        self.analyzer.advance();
        self.outfile.write_all(b"</classVarDec>\n").unwrap();
    }

    pub fn compile_subroutine(&mut self) {
        self.outfile.write_all(b"<subroutineDec>\n").unwrap();
        write_tag_string(&self.analyzer, &mut self.outfile); // Write function keyword

        self.analyzer.advance();
        if !(self.analyzer.token_type() == TokenType::Identifier || self.analyzer.token_type() == TokenType::Keyword) {
            panic!("No return type");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Identifier {
            panic!("No function name");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '(' {
            panic!("Missing parameter list");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.analyzer.advance();
        self.compile_parameter_list();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != ')' {
            panic!("Missing closing parenthesis");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.outfile.write_all(b"<subroutineBody>\n").unwrap();
        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '{' {
            panic!("Missing function opening brace");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.analyzer.advance();
        while self.analyzer.token_type() == TokenType::Keyword && self.analyzer.key_word().unwrap() == Keyword::Var {
            self.compile_var_dec();
        }

        self.compile_statements();
        write_tag_string(&self.analyzer, &mut self.outfile); // Write closing brace
        self.analyzer.advance();

        self.outfile.write_all(b"</subroutineBody>\n").unwrap();
        self.outfile.write_all(b"</subroutineDec>\n").unwrap();
    }

    pub fn compile_parameter_list(&mut self) {
        self.outfile.write_all(b"<parameterList>\n").unwrap();
        while !(self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == ')') {
            write_tag_string(&self.analyzer, &mut self.outfile);
            self.analyzer.advance();
        }
        self.outfile.write_all(b"</parameterList>\n").unwrap();
    }

    pub fn compile_var_dec(&mut self) {
        self.outfile.write_all(b"<varDec>\n").unwrap();
        fn end_of_var(analyzer: &JackAnalyzer) -> bool {
            if analyzer.token_type() != TokenType::Symbol {
                return false;
            }
            analyzer.symbol() == ';'
        }

        while !end_of_var(&self.analyzer) {
            write_tag_string(&self.analyzer, &mut self.outfile);
            self.analyzer.advance();
        }
        write_tag_string(&self.analyzer, &mut self.outfile); // Write semicolon
        self.analyzer.advance();
        self.outfile.write_all(b"</varDec>\n").unwrap();
    }

    pub fn compile_statements(&mut self) {
        self.outfile.write_all(b"<statements>\n").unwrap();
        while !(self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == '}') {
            if self.analyzer.token_type() != TokenType::Keyword {
                panic!("Statement must begin with keyword");
            }

            match self.analyzer.key_word().unwrap() {
                Keyword::Let => self.compile_let(),
                Keyword::If => self.compile_if(),
                Keyword::While => self.compile_while(),
                Keyword::Do => self.compile_do(),
                Keyword::Return => self.compile_return(),
                other => panic!("Invalid keyword at start of statement: ".to_string() +
                           keyword_to_str(&other)),
            };
        }
        self.outfile.write_all(b"</statements>\n").unwrap();
    }

    pub fn compile_do(&mut self) {
        self.outfile.write_all(b"<doStatement>\n").unwrap();
        while !(self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == '(') {
            write_tag_string(&self.analyzer, &mut self.outfile); // Write do keyword
            self.analyzer.advance();
        }
        write_tag_string(&self.analyzer, &mut self.outfile); // Write (
        self.analyzer.advance();

        self.compile_expression_list();

        write_tag_string(&self.analyzer, &mut self.outfile); // Write )
        self.analyzer.advance();
        write_tag_string(&self.analyzer, &mut self.outfile); // Write semicolon
        self.analyzer.advance();
        self.outfile.write_all(b"</doStatement>\n").unwrap();
    }

    pub fn compile_let(&mut self) {
        self.outfile.write_all(b"<letStatement>\n").unwrap();
        write_tag_string(&self.analyzer, &mut self.outfile); // Write let keyword
        self.analyzer.advance();
        write_tag_string(&self.analyzer, &mut self.outfile); // Write variable name
        self.analyzer.advance();
        
        // TODO: handle array element assignment
        if self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == '[' {
            write_tag_string(&self.analyzer, &mut self.outfile); // Write [
            self.analyzer.advance();

            self.compile_expression();
            
            write_tag_string(&self.analyzer, &mut self.outfile); // Write ]
            self.analyzer.advance();
        }
        
        write_tag_string(&self.analyzer, &mut self.outfile); // Write =
        self.analyzer.advance();

        self.compile_expression();

        //println!("this is my semicolon: {}", self.analyzer.symbol());
        write_tag_string(&self.analyzer, &mut self.outfile); // Write semicolon
        self.analyzer.advance();
        self.outfile.write_all(b"</letStatement>\n").unwrap();
    }

    pub fn compile_while(&mut self) {
        self.outfile.write_all(b"<whileStatement>\n").unwrap();
        write_tag_string(&self.analyzer, &mut self.outfile);
        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '(' {
            panic!("Missing expression for while loop");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.analyzer.advance();
        self.compile_expression();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != ')' {
            panic!("Missing closing parenthesis for while expression");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '{' {
            panic!("Missing opening brace on while loop");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.analyzer.advance();
        self.compile_statements();
        write_tag_string(&self.analyzer, &mut self.outfile); // Write closing brace
        self.analyzer.advance();
        self.outfile.write_all(b"</whileStatement>\n").unwrap();
    }

    pub fn compile_return(&mut self) {
        self.outfile.write_all(b"<returnStatement>\n").unwrap();
        write_tag_string(&self.analyzer, &mut self.outfile); // Write return keyword
        self.analyzer.advance();
        if !(self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == ';') {
            self.compile_expression();
        }
        write_tag_string(&self.analyzer, &mut self.outfile); // Write semicolon
        self.analyzer.advance();
        self.outfile.write_all(b"</returnStatement>\n").unwrap();
    }

    pub fn compile_if(&mut self) {
        self.outfile.write_all(b"<ifStatement>\n").unwrap();
        write_tag_string(&self.analyzer, &mut self.outfile);
        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '(' {
            panic!("Missing expression for if statement");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.analyzer.advance();
        self.compile_expression();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != ')' {
            panic!("Missing closing parenthesis for if expression");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '{' {
            panic!("Missing opening brace on if expression");
        }
        write_tag_string(&self.analyzer, &mut self.outfile);

        self.analyzer.advance();
        self.compile_statements();
        write_tag_string(&self.analyzer, &mut self.outfile); // Write closing brace
        self.analyzer.advance();
        self.outfile.write_all(b"</ifStatement>\n").unwrap();
    }

    pub fn compile_expression(&mut self) {
        let ops = [
            '+', '-', '*', '/', '&',
            '|', '<', '>', '=',
        ];

        self.outfile.write_all(b"<expression>\n").unwrap();
        self.compile_term();
        while self.analyzer.token_type() == TokenType::Symbol && ops.contains(&self.analyzer.symbol()) {
            write_tag_string(&self.analyzer, &mut self.outfile);
            self.analyzer.advance();

            self.compile_term();
        }
        self.outfile.write_all(b"</expression>\n").unwrap();
    }

    pub fn compile_term(&mut self) {
        self.outfile.write_all(b"<term>\n").unwrap();
        let simple_ops = [
            TokenType::IntConst, TokenType::StringConst, TokenType::Keyword,
        ];

        let lookahead_symbols = [
            '(', '[',
        ];

        // Just write the expression if it's simple
        let current_token_type = self.analyzer.token_type();
        if simple_ops.contains(&current_token_type) {
            write_tag_string(&self.analyzer, &mut self.outfile);
            self.analyzer.advance();
        }
        // Parse negation or inversion
        else if current_token_type == TokenType::Symbol && ['-', '~'].contains(&self.analyzer.symbol()) {
            write_tag_string(&self.analyzer, &mut self.outfile);
            self.analyzer.advance();
            self.compile_term();
        }
        // Parse sub-expression in ()
        else if current_token_type == TokenType::Symbol && self.analyzer.symbol() == '(' {
                write_tag_string(&self.analyzer, &mut self.outfile); // Write (
                self.analyzer.advance();

                self.compile_expression();
                
                write_tag_string(&self.analyzer, &mut self.outfile); // Write )
                self.analyzer.advance();
        } else {
            // Parse expression that requires lookahead
            while self.analyzer.token_type() == TokenType::Identifier || (self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol () == '.') {
                //self.outfile.write_all(b"yolo");
                write_tag_string(&self.analyzer, &mut self.outfile);
                self.analyzer.advance();
            }

            // Check if the next symbol is the start or end of special case
            let next_symbol = self.analyzer.symbol();
            if lookahead_symbols.contains(&next_symbol) {
                write_tag_string(&self.analyzer, &mut self.outfile);
                self.analyzer.advance();
                // See if the statement is an array, function call or unary operator
                match next_symbol {
                    '(' => self.compile_expression_list(), // Function
                    '[' => self.compile_expression(),      // Array,
                    _ => (),
                }

                if next_symbol == '(' || next_symbol == '[' {
                    write_tag_string(&self.analyzer, &mut self.outfile);
                    self.analyzer.advance();
                }
            }
        }
        self.outfile.write_all(b"</term>\n").unwrap();
    }

    pub fn compile_expression_list(&mut self) {
        self.outfile.write_all(b"<expressionList>\n").unwrap();
        while !(self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == ')') {
            self.compile_expression();
            if self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == ',' {
                write_tag_string(&self.analyzer, &mut self.outfile);
                self.analyzer.advance();
            }
        }
        self.outfile.write_all(b"</expressionList>\n").unwrap();
    }
}
