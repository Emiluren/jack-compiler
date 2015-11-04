mod jack_analyzer;
mod compilation_engine;
mod xml_output;

use jack_analyzer::*;
use compilation_engine::*;
use xml_output::*;

use std::env;
use std::fs::File;
use std::io::prelude::*;

fn write_token_file(infile: &String, outname: &String) {
    let mut analyzer = JackAnalyzer::new(infile);
    let mut outfile = File::create(outname).unwrap();
    outfile.write_all(b"<tokens>\n").unwrap();

    while analyzer.has_more_tokens() {
        analyzer.advance();
        write_tag_string(&analyzer, &mut outfile);
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
            let localname = filename.split('/').last().unwrap();
            let outname = localname.split('.').next().unwrap().to_string() + ".xml";
            println!("Compiling {} to {}", filename, outname);
            //write_token_file(filename, &outname);
            
            let mut compiler = CompilationEngine::new(filename, &outname);
            compiler.compile_class();
            current_file += 1;
        }
    }
}
