use language::kinds::Kinds;

pub struct Document;

pub struct Node {
    pub kind: Kinds,
    pub operation: String,
    pub name: Option<String>,
    pub variable_definitions: Option<Vec<VariableDefinition>>,
    pub directives: Vec<Directive>,
    pub selection_set: SelectionSet,
    pub loc: Option<Location>
}

pub struct VariableDefinition;
pub struct Directive;
pub struct SelectionSet {
    pub kind: Kinds,
    pub selections: Vec<Selection>,
    pub loc: Option<Location>,
}
pub struct Selection {
    pub kind: Kinds,
    pub alias: Option<NamedType>,
    pub name: NamedType,
    pub arguments: Vec<Argument>,
    pub directives: Vec<Directive>,
    pub selection_set: Option<SelectionSet>,
    pub loc: Option<Location>
}
pub struct Location;
pub struct NamedType {
    pub kind: Kinds,
    pub value: Option<String>,
    pub loc: Option<Location>
}
pub struct Argument;
