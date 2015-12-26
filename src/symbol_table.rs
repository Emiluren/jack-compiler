use jack_analyzer::Keyword;

use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    None,
    Static,
    Field,
    Arg,
    Var,
}

struct TableEntry {
    type_name: String,
    kind: Kind,
    index: i32,
}

pub struct SymbolTable {
    class_symbols: HashMap<String, TableEntry>,
    function_symbols: HashMap<String, TableEntry>,
    
    static_index: i32,
    field_index: i32,
    arg_index: i32,
    var_index: i32,
}

pub fn keyword_to_kind(keyword: Keyword) -> Kind {
    match keyword {
        Keyword::Static => Kind::Static,
        Keyword::Field => Kind::Field,
        Keyword::Var => Kind::Var,
        _ => Kind::None,
    }
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            class_symbols: HashMap::new(),
            function_symbols: HashMap::new(),

            static_index: 0,
            field_index: 0,
            arg_index: 0,
            var_index: 0,
        }
    }
    
    pub fn start_subroutine(&mut self) {
        self.function_symbols.clear();
        self.arg_index = 0;
        self.var_index = 0;
    }

    pub fn define(&mut self, name: String, t: String, k: Kind) {
        match k {
            Kind::Static => {
                self.class_symbols.insert(name, TableEntry {type_name: t, kind: k, index: self.static_index});
                self.static_index += 1;
            }
            Kind::Field => {
                self.class_symbols.insert(name, TableEntry {type_name: t, kind: k, index: self.field_index});
                self.field_index += 1;
            }
            Kind::Arg => {
                self.function_symbols.insert(name, TableEntry {type_name: t, kind: k, index: self.arg_index});
                self.arg_index += 1;
            }
            Kind::Var => {
                self.class_symbols.insert(name, TableEntry {type_name: t, kind: k, index: self.var_index});
                self.var_index += 1;
            }
            Kind::None => (),
        }
    }

    pub fn var_count(&self, kind: Kind) -> i32 {
        match kind {
            Kind::Static => self.static_index,
            Kind::Field => self.field_index,
            Kind::Arg => self.arg_index,
            Kind::Var => self.var_index,
            _ => panic!("No symbols of type None!")
        }
    }

    pub fn kind_of(&self, name: &String) -> Kind {
        if self.function_symbols.contains_key(name) {
            self.function_symbols.get(name).unwrap().kind
        } else if self.class_symbols.contains_key(name) {
            self.class_symbols.get(name).unwrap().kind
        } else {
            Kind::None
        }
    }

    pub fn type_of(&self, name: &String) -> String {
        if self.function_symbols.contains_key(name) {
            self.function_symbols.get(name).unwrap().type_name.clone()
        } else {
            self.class_symbols.get(name).unwrap().type_name.clone()
        }
    }

    pub fn index_of(&self, name: &String) -> i32 {
        if self.function_symbols.contains_key(name) {
            self.function_symbols.get(name).unwrap().index
        } else {
            self.class_symbols.get(name).unwrap().index
        }
    }
}
