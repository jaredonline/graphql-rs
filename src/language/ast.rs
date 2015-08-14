use language::kinds::Kinds;
use language::lexer::Source;

use std::fmt;
use std::io::Write;

#[derive(PartialEq)]
pub struct Document {
    pub kind: Kinds,
    pub loc: Option<Location>,
    pub definitions: Vec<Definition>
}

impl fmt::Debug for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Document")
            .field("kind", &self.kind)
            .field("loc", &self.loc)
            .field("definitions", &self.definitions)
            .finish()
    }
}

#[derive(PartialEq, Debug)]
pub enum Definition {
    Operation {
        kind: Kinds,
        operation: String,
        name: Option<Name>,
        variable_definitions: Option<Vec<VariableDefinition>>,
        directives: Vec<Directive>,
        selection_set: SelectionSet,
        loc: Option<Location>
    },
    Fragment {
        kind: Kinds,
        name: Name,
        type_condition: Type,
        directives: Option<Vec<Directive>>,
        selection_set: SelectionSet,
        loc: Option<Location>
    }
}

#[derive(PartialEq, Debug)]
pub struct VariableDefinition {
    pub kind: Kinds,
    pub variable: Value,
    pub var_type: Type,
    pub default_value: Option<Value>,
    pub loc: Option<Location>,
}
#[derive(PartialEq, Debug)]
pub struct Directive {
    pub kind: Kinds,
    pub name: Name,
    pub arguments: Option<Vec<Argument>>,
    pub loc: Option<Location>,
}
#[derive(PartialEq, Debug)]
pub struct SelectionSet {
    pub kind: Kinds,
    pub selections: Vec<Selection>,
    pub loc: Option<Location>,
}

#[derive(PartialEq, Debug)]
pub enum Selection {
    Field {
        kind: Kinds,
        alias: Option<Name>,
        name: Name,
        arguments: Vec<Argument>,
        directives: Vec<Directive>,
        selection_set: Option<SelectionSet>,
        loc: Option<Location>
    },
    FragmentSpread {
        kind: Kinds,
        name: Name,
        directives: Option<Vec<Directive>>,
        loc: Option<Location>
    },
    InlineFragment {
        kind: Kinds,
        type_condition: Type,
        directives: Option<Vec<Directive>>,
        selection_set: SelectionSet,
        loc: Option<Location>,
    }
}
#[derive(PartialEq, Clone)]
pub struct Location {
    pub start: usize,
    pub end: usize,
    pub source: Option<Source>
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.start, self.end)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Name {
    pub kind: Kinds,
    pub value: String,
    pub loc: Option<Location>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Type {
    Named { kind: Kinds, name: Name, loc: Option<Location> },
    List { kind: Kinds, t_type: Box<Type>, loc: Option<Location> },
    NonNull { kind: Kinds, t_type: Box<Type>, loc: Option<Location> },
}

#[derive(PartialEq, Debug)]
pub struct Argument {
    pub kind: Kinds,
    pub name: Name,
    pub value: Value,
    pub loc: Option<Location>
}

#[derive(PartialEq, Debug)]
pub enum Value {
    IntValue { kind: Kinds, value: String, loc: Option<Location> },
    FloatValue { kind: Kinds, value: String, loc: Option<Location> },
    StringValue { kind: Kinds, value: String, loc: Option<Location> },
    BooleanValue { kind: Kinds, value: bool, loc: Option<Location> },
    EnumValue { kind: Kinds, value: String, loc: Option<Location> },
    ArrayValue { kind: Kinds, values: Vec<Value>, loc: Option<Location> },
    ObjectValue { kind: Kinds, fields: Vec<ObjectField>, loc: Option<Location> },
    VariableValue { kind: Kinds, name: Name, loc: Option<Location> },
}

#[derive(PartialEq, Debug)]
pub struct ObjectField {
    pub kind: Kinds,
    pub name: Name,
    pub value: Value,
    pub loc: Option<Location>
}
