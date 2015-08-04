use language::kinds::Kinds;
use language::lexer::Source;

#[derive(PartialEq, Debug)]
pub struct Document {
    pub kind: Kinds,
    pub loc: Option<Location>,
    pub definitions: Vec<Node>
}

#[derive(PartialEq, Debug)]
pub struct Node {
    pub kind: Kinds,
    pub operation: String,
    pub name: Option<String>,
    pub variable_definitions: Option<Vec<VariableDefinition>>,
    pub directives: Vec<Directive>,
    pub selection_set: SelectionSet,
    pub loc: Option<Location>
}

#[derive(PartialEq, Debug)]
pub struct VariableDefinition;
#[derive(PartialEq, Debug)]
pub struct Directive;
#[derive(PartialEq, Debug)]
pub struct SelectionSet {
    pub kind: Kinds,
    pub selections: Vec<Selection>,
    pub loc: Option<Location>,
}
#[derive(PartialEq, Debug)]
pub struct Selection {
    pub kind: Kinds,
    pub alias: Option<NamedType>,
    pub name: NamedType,
    pub arguments: Vec<Argument>,
    pub directives: Vec<Directive>,
    pub selection_set: Option<SelectionSet>,
    pub loc: Option<Location>
}
#[derive(PartialEq, Debug)]
pub struct Location {
    pub start: usize,
    pub end: usize,
    pub source: Option<Source>
}
#[derive(PartialEq, Debug)]
pub struct NamedType {
    pub kind: Kinds,
    pub value: Option<String>,
    pub loc: Option<Location>
}
#[derive(PartialEq, Debug)]
pub struct Argument;
