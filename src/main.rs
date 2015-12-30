mod jack_analyzer;
mod compilation_engine;
mod xml_output;
mod symbol_table;
mod vm_writer;

use compilation_engine::*;

use std::env;
use std::io::prelude::*;

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut current_file: usize = 1;
    if args.len() < 2 {
        println!("usage: jackcompiler files");
    } else {
        while current_file < args.len() {
            let filename = &args[current_file];
            
            let outname = filename.split('.').next().unwrap().to_string() + ".vm";
            println!("Compiling {} to {}", filename, outname);
            
            let mut compiler = CompilationEngine::new(filename, &outname);
            compiler.compile_class();
            current_file += 1;
        }
    }
}
