use jack_analyzer::*;
use vm_writer::*;
use symbol_table::*;
use xml_output::keyword_to_str;
use xml_output::make_tag_string;

use std::path::Path;

pub struct CompilationEngine {
    analyzer: JackAnalyzer,
    vm_writer: VMWriter,
    symbol_table: SymbolTable,
    class_name: String,
    label_num: i32,
}

fn kind_to_segment(kind: Kind) -> Segment {
    match kind {
        Kind::Static => Segment::Static,
        Kind::Field => Segment::This,
        Kind::Arg => Segment::Arg,
        Kind::Var => Segment::Local,
        _ => panic!("None kind no segment"),
    }
}

fn symbol_to_command(sym: char) -> Command {
    match sym {
        '+' => Command::Add,
        '-' => Command::Sub,
        '&' => Command::And,
        '|' => Command::Or,
        '<' => Command::Lt,
        '>' => Command::Gt,
        '=' => Command::Eq,
        _ => panic!(format!("symbol {} does not have an associated command", sym))
    }
}

// TODO: maybe make everything more rusty with results instead of panics
// TODO: have some way of reporting line number on errors (maybe count lines in JackAnalyzer)

impl CompilationEngine {
    pub fn new(infile: &Path, outfile: &Path) -> CompilationEngine {
        CompilationEngine {
            analyzer: JackAnalyzer::new(infile),
            vm_writer: VMWriter::new(outfile),
            symbol_table: SymbolTable::new(),
            class_name: String::new(),
            label_num: 0,
        }
    }
    
    fn gen_label_num(&mut self) -> String {
        self.label_num += 1;
        self.label_num.to_string()
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
        self.class_name = self.analyzer.identifier();

        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '{' {
            panic!("Missing opening brace");
        }

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
                _ => panic!("Unknown token inside class: "),
            };
        }
    }
    
    fn compile_generic_var_dec(&mut self) {
        // Check if it's a static variable, field, or local variable
        let kind = keyword_to_kind(self.analyzer.key_word().unwrap());
        self.analyzer.advance();
        
        // Get the type of the variable
        let type_name = self.analyzer.identifier();
        self.analyzer.advance();
        
        // Define all variables
        while !(self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == ';') {
            // Skip commas
            if self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == ',' {
                self.analyzer.advance();
            }
            // Get the name of the variable
            let name = self.analyzer.identifier();
            self.symbol_table.define(&name, &type_name, kind);
            self.analyzer.advance();
        }
    }

    pub fn compile_class_var_dec(&mut self) {
        self.compile_generic_var_dec();
        // Skip semicolon
        self.analyzer.advance();
    }

    pub fn compile_subroutine(&mut self) {
        // Clear symbol table
        self.symbol_table.start_subroutine();

        let subroutine_type = self.analyzer.key_word().unwrap();
        self.analyzer.advance();

        if !(self.analyzer.token_type() == TokenType::Identifier || self.analyzer.token_type() == TokenType::Keyword) {
            panic!("No return type");
        }
        self.analyzer.advance();

        if self.analyzer.token_type() != TokenType::Identifier {
            panic!("No function name");
        }
        let fn_name = format!("{}.{}", self.class_name, self.analyzer.identifier());
        self.analyzer.advance();

        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '(' {
            panic!("Missing parameter list");
        }
        self.analyzer.advance();

        self.compile_parameter_list();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != ')' {
            panic!("Missing closing parenthesis");
        }

        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '{' {
            panic!("Missing function opening brace");
        }

        // Parse local variable declarations
        self.analyzer.advance();
        while self.analyzer.token_type() == TokenType::Keyword && self.analyzer.key_word().unwrap() == Keyword::Var {
            self.compile_var_dec();
        }
        
        let n_local = self.symbol_table.var_count(Kind::Var);
        self.vm_writer.write_function(&fn_name, n_local);

        // Set up this pointer
        if subroutine_type == Keyword::Constructor {
            // Allocate space for the object
            self.vm_writer.write_push(Segment::Const, self.symbol_table.var_count(Kind::Field));
            self.vm_writer.write_call("Memory.alloc", 1);
            self.vm_writer.write_pop(Segment::Pointer, 0);
        } else if subroutine_type == Keyword::Method {
            // Set this pointer to current object
            self.vm_writer.write_push(Segment::Arg, 0);
            self.vm_writer.write_pop(Segment::Pointer, 0);
        }

        // Write main body of subroutine
        self.compile_statements();

        // Skip closing brace
        self.analyzer.advance();
    }

    pub fn compile_parameter_list(&mut self) {
        while !(self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == ')') {
            // Skip commas between arguments
            if self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == ',' {
                self.analyzer.advance();
            }
            // Get the type of the variable
            let type_name = self.analyzer.identifier();
            self.analyzer.advance();

            // Get the name of the variable
            let name = self.analyzer.identifier();
            self.symbol_table.define(&name, &type_name, Kind::Arg);
            self.analyzer.advance();
        }
    }

    pub fn compile_var_dec(&mut self) {
        self.compile_generic_var_dec();
        // Skip semicolon
        self.analyzer.advance();
    }

    pub fn compile_statements(&mut self) {
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
    }
    
    fn compile_function_call(&mut self, sym: char, name1: String) {
        //println!("at start of function call. state = {}, sym = {}", make_tag_string(&self.analyzer), sym.to_string());
        let mut n_args = 0;
        let full_name = if sym == '.' {
            // name1 is a the name of a class or an object
            let f_name = self.analyzer.identifier();
            self.analyzer.advance();
            let kind = self.symbol_table.kind_of(&name1);
            // If it's a static function we can just write the class name, otherwise we need to find it
            let class_name = if kind == Kind::None {
                name1
            } else {
                // Push the object to the stack
                self.vm_writer.write_push(kind_to_segment(kind), self.symbol_table.index_of(&name1));
                n_args += 1;
                self.symbol_table.type_of(&name1)
            };
        
            if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '(' {
                panic!(format!("Expected ( after function name. found {} instead", make_tag_string(&self.analyzer)));
            }
            self.analyzer.advance();
            
            class_name + "." + &f_name
        } else if sym == '(' {
            // name1 is the function name
            // Local function, push this to stack
            self.vm_writer.write_push(Segment::Pointer, 0);
            n_args += 1;
            format!("{}.{}", self.class_name, name1)
        } else {
            panic!("Expected on of . and ( after identifier in function call")
        };

        // Push parameters
        n_args += self.compile_expression_list();

        // Skip )
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != ')' {
            panic!(format!("Expected ) after function name. found {} instead", make_tag_string(&self.analyzer)));
        }
        self.analyzer.advance();

        self.vm_writer.write_call(&full_name, n_args);
    }

    pub fn compile_do(&mut self) {
        // Skip do
        self.analyzer.advance();

        let name1 = self.analyzer.identifier();
        self.analyzer.advance();

        if self.analyzer.token_type() != TokenType::Symbol {
            panic!("Symbol . or ( expected after identifier in do statement")
        }
        let sym = self.analyzer.symbol();
        self.analyzer.advance();

        // Check what kind of function it is, call it and push return value to stack
        self.compile_function_call(sym, name1);

        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != ';' {
            panic!("Expected ; at end of do statement");
        }
        self.analyzer.advance();

        // Ignore return value
        self.vm_writer.write_pop(Segment::Temp, 0);
    }

    pub fn compile_let(&mut self) {
        // Skip let keyword
        self.analyzer.advance();

        // Parse variable name
        let var_name = self.analyzer.identifier();
        self.analyzer.advance();
        
        let kind = self.symbol_table.kind_of(&var_name);
        let seg = kind_to_segment(kind);
        let index = self.symbol_table.index_of(&var_name);

        // handle array element assignment
        if self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == '[' {
            // Skip [
            self.analyzer.advance();

            // Place index on stack
            self.compile_expression();
            
            // Skip ] TODO: check
            self.analyzer.advance();
        
            // Skip = TODO: check
            self.analyzer.advance();

            // Calculate address
            self.vm_writer.write_push(seg, index);
            self.vm_writer.write_arithmetic(Command::Add);

            // Place expression result on stack and do the assignment
            // temp stuff is required since compile_expression might change pointer 1
            self.compile_expression();
            self.vm_writer.write_pop(Segment::Temp, 0);
            self.vm_writer.write_pop(Segment::Pointer, 1);
            self.vm_writer.write_push(Segment::Temp, 0);
            self.vm_writer.write_pop(Segment::That, 0)
        } else {
            // Skip = TODO: check
            self.analyzer.advance();

            self.compile_expression();

            self.vm_writer.write_pop(seg, index);
        }

        // Skip semicolon
        self.analyzer.advance();
    }

    pub fn compile_while(&mut self) {
        // Skip while keyword
        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '(' {
            panic!("Missing expression for while loop");
        }
        // Skip (
        self.analyzer.advance();

        // TODO: add variable for num
        let while_label = "while".to_string() + &self.gen_label_num();
        let end_label = &format!("{}end", while_label);
        self.vm_writer.write_label(&while_label);
        // Calculate expression and check if loop should be continued
        self.compile_expression();
        self.vm_writer.write_arithmetic(Command::Not);
        self.vm_writer.write_if(&end_label);

        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != ')' {
            panic!("Missing closing parenthesis for while expression");
        }
        // Skip )
        self.analyzer.advance();

        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '{' {
            panic!("Missing opening brace on while loop");
        }
        // Skip {
        self.analyzer.advance();

        // Compile statements inside loop
        self.compile_statements();

        // Skip } TODO: check
        self.analyzer.advance();

        self.vm_writer.write_goto(&while_label);
        self.vm_writer.write_label(&end_label)
    }

    pub fn compile_return(&mut self) {
        // Skip return keyword
        self.analyzer.advance();
        if !(self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == ';') {
            self.compile_expression();
        } else {
            self.vm_writer.write_push(Segment::Const, 0);
        }
        // Skip ;
        self.analyzer.advance();
        
        self.vm_writer.write_return();
    }

    pub fn compile_if(&mut self) {
        // Skip if keyword
        self.analyzer.advance();
        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '(' {
            panic!("Missing expression for if statement");
        }
        // Skip (
        self.analyzer.advance();

        let if_label = "if".to_string() + &self.gen_label_num();
        let end_label = format!("{}end", if_label);
        let else_label = format!("{}else", if_label);
        
        // Push result of expression to stack and skip to else if not true
        // TODO: do not skip to else if it does not exist
        self.compile_expression();
        self.vm_writer.write_arithmetic(Command::Not);
        self.vm_writer.write_if(&else_label);

        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != ')' {
            panic!("Missing closing parenthesis for if expression");
        }
        // Skip )
        self.analyzer.advance();

        if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '{' {
            panic!("Missing opening brace on if statement");
        }
        // Skip {
        self.analyzer.advance();

        // Write if part
        self.compile_statements();
        self.vm_writer.write_goto(&end_label);

        // Skip closing brace
        self.analyzer.advance();

        // Manage else part
        self.vm_writer.write_label(&else_label);
        if self.analyzer.token_type() == TokenType::Keyword && self.analyzer.key_word() == Some(Keyword::Else) {
            // Skip else keyword
            self.analyzer.advance();

            if self.analyzer.token_type() != TokenType::Symbol || self.analyzer.symbol() != '{' {
                panic!("Missing opening brace on else statement");
            }
            self.analyzer.advance();
            
            // Compile statements in else part
            self.compile_statements();

            // Skip closing brace }
            self.analyzer.advance();
        }
        self.vm_writer.write_label(&end_label);
    }

    pub fn compile_expression(&mut self) {
        let ops = [
            '+', '-', '*', '/', '&',
            '|', '<', '>', '=',
        ];

        // Push first term to stack
        //println!("current fterm {}", make_tag_string(&self.analyzer));
        self.compile_term();
        while self.analyzer.token_type() == TokenType::Symbol && ops.contains(&self.analyzer.symbol()) {
            //println!("yolo");
            let sym = self.analyzer.symbol();
            //println!("current symbol {}", make_tag_string(&self.analyzer));
            self.analyzer.advance();

            // Push new term to stack and do calculation
            self.compile_term();
            if sym != '*' && sym != '/' {
                self.vm_writer.write_arithmetic(symbol_to_command(sym));
            } else if sym == '*' {
                self.vm_writer.write_call("Math.multiply", 2);
            } else if sym == '/' {
                self.vm_writer.write_call("Math.divide", 2);
            }
        }
    }

    pub fn compile_term(&mut self) {
        //println!("current term {}", make_tag_string(&self.analyzer));
        let current_token_type = self.analyzer.token_type();
        // Push constants directly
        if current_token_type == TokenType::IntConst {
            self.vm_writer.write_push(Segment::Const, self.analyzer.int_val());
            self.analyzer.advance();
        } else if current_token_type == TokenType::StringConst {
            // Create a new string object and append all the characters
            let string = self.analyzer.string_val();
            self.vm_writer.write_push(Segment::Const, string.len() as i32);
            self.vm_writer.write_call("String.new", 1);
            for c in string.chars() {
                self.vm_writer.write_push(Segment::Const, c as i32);
                self.vm_writer.write_call("String.appendChar", 2);
            }
            self.analyzer.advance();
        } else if current_token_type == TokenType::Keyword {
            let keyword = self.analyzer.key_word().unwrap();
            if keyword == Keyword::This {
                self.vm_writer.write_push(Segment::Pointer, 0);
            } else if keyword == Keyword::True {
                self.vm_writer.write_push(Segment::Const, 1);
                self.vm_writer.write_arithmetic(Command::Neg);
            } else {
                let val = match keyword {
                    Keyword::False => 0,
                    Keyword::Null => 0,
                    _ => panic!("Invalid keyword in expression")
                };
                self.vm_writer.write_push(Segment::Const, val);
            }
            self.analyzer.advance();
        }
        // Parse negation or inversion
        else if current_token_type == TokenType::Symbol && '-' == self.analyzer.symbol() {
            self.analyzer.advance();
            self.compile_term();
            self.vm_writer.write_arithmetic(Command::Neg);
        } else if current_token_type == TokenType::Symbol && '~' == self.analyzer.symbol() {
            self.analyzer.advance();
            self.compile_term();
            self.vm_writer.write_arithmetic(Command::Not);
        }
        // Parse sub-expression in ()
        else if current_token_type == TokenType::Symbol && self.analyzer.symbol() == '(' {
                // Skip (
                self.analyzer.advance();

                self.compile_expression();
                
                // Skip )
                self.analyzer.advance();
        } else {
            // Parse expression that requires variable, function call or array
            if self.analyzer.token_type() != TokenType::Identifier {
                panic!(format!("Unexpected token inside expression term {}", make_tag_string(&self.analyzer)));
            }
            
            let name1 = self.analyzer.identifier();
            self.analyzer.advance();

            // Expecing one of [ . or something new
            if self.analyzer.token_type() != TokenType::Symbol {
                panic!("Weird shit is going on inside term. Expected symbol");
            }

            // Check if it's a function call or array
            let next_symbol = self.analyzer.symbol();
            if next_symbol == '(' || next_symbol == '.' {
                self.analyzer.advance();
                self.compile_function_call(next_symbol, name1);
            } else if next_symbol == '[' {
                self.analyzer.advance();
                // Calculate address
                self.compile_expression();

                // Skip ]
                self.analyzer.advance();
                
                self.vm_writer.write_push(kind_to_segment(
                    self.symbol_table.kind_of(&name1)), self.symbol_table.index_of(&name1));
                self.vm_writer.write_arithmetic(Command::Add);
                self.vm_writer.write_pop(Segment::Pointer, 1);
                // Push content to stack
                self.vm_writer.write_push(Segment::That, 0);
            } else {
                // It's a simple variable, push it (like a boss!)
                self.vm_writer.write_push(kind_to_segment(
                    self.symbol_table.kind_of(&name1)), self.symbol_table.index_of(&name1));
            }
        }
    }

    pub fn compile_expression_list(&mut self) -> i32 {
        let mut n = 0;
        while !(self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == ')') {
            self.compile_expression();
            if self.analyzer.token_type() == TokenType::Symbol && self.analyzer.symbol() == ',' {
                self.analyzer.advance();
            }
            n += 1;
        }
        n
    }
}
