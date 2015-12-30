use std::fs::File;
use std::io::prelude::*;

#[derive(Clone, Copy)]
pub enum Segment {
    Const,
    Arg,
    Local,
    Static,
    This,
    That,
    Pointer,
    Temp
}

#[derive(Clone, Copy)]
pub enum Command {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
}

pub struct VMWriter {
    outfile: File,
}

fn segment_string(seg: Segment) -> &'static str {
    match seg {
        Segment::Const => "const",
        Segment::Local => "local",
        Segment::Arg => "arg",
        Segment::Static => "static",
        Segment::This => "this",
        Segment::That => "that",
        Segment::Pointer => "pointer",
        Segment::Temp => "temp",
    }
}

fn command_string(com: Command) -> &'static str {
    match com {
        Command::Add => "add",
        Command::Sub => "sub",
        Command::Neg => "neg",
        Command::Eq => "eq",
        Command::Gt => "gt",
        Command::Lt => "lt",
        Command::And => "and",
        Command::Or => "or",
        Command::Not => "not",
    }
}

impl VMWriter {
    pub fn new(filename: &str) -> VMWriter {
        VMWriter {
            outfile: File::create(filename).unwrap(),
        }
    }
    
    fn write_string(&mut self, data: String) {
        self.outfile.write_all(data.as_bytes()).unwrap();
    }

    pub fn write_push(&mut self, seg: Segment, index: i32) {
        self.write_string(format!("push {0} {1}\n", segment_string(seg), index));
    }

    pub fn write_pop(&mut self, seg: Segment, index: i32) {
        self.write_string(format!("pop {0} {1}\n", segment_string(seg), index));
    }

    pub fn write_arithmetic(&mut self, com: Command) {
        self.write_string(format!("{0}\n", command_string(com)));
    }

    pub fn write_label(&mut self, label: &str) {
        self.write_string(format!("label {0}\n", label));
    }

    pub fn write_goto(&mut self, label: &str) {
        self.write_string(format!("goto {0}\n", label))
    }

    pub fn write_if(&mut self, label: &str) {
        self.write_string(format!("if-goto {0}\n", label))
    }

    pub fn write_call(&mut self, name: &str, n_args: i32) {
        self.write_string(format!("call {0} {1}\n", name, n_args))
    }

    pub fn write_function(&mut self, name: &str, n_args: i32) {
        self.write_string(format!("function {0} {1}\n", name, n_args))
    }

    pub fn write_return(&mut self) {
        self.write_string("return\n".to_string());
    }

    pub fn close() {
        // TODO: implement
    }
}
