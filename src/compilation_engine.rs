use jack_analyzer::*;

pub struct CompilationEngine {
    analyzer: JackAnalyzer,
}

impl CompilationEngine {
    pub fn new(infile: &str, outfile: &str) -> CompilationEngine {
        CompilationEngine {
            analyzer: JackAnalyzer::new(infile),
        }
    }

    pub fn compile_class() {
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
