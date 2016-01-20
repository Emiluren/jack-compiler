mod jack_analyzer;
mod compilation_engine;
mod xml_output;
mod symbol_table;
mod vm_writer;

use compilation_engine::*;

use std::env;
use std::path::Path;

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut current_file: usize = 1;
    if args.len() < 2 {
        println!("usage: jackcompiler files");
    } else {
        // Compile every file
        while current_file < args.len() {
            let filename = &args[current_file];
            let path = Path::new(filename);
            let outfile = path.with_extension("vm");
            
            println!("Compiling {} to {}", filename, outfile.display());
            
            let mut compiler = CompilationEngine::new(path, &outfile);
            compiler.compile_class();
            current_file += 1;
        }
    }
}
