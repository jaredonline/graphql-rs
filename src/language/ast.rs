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
#[derive(PartialEq, Debug, Clone)]
pub struct Location {
    pub start: usize,
    pub end: usize,
    pub source: Option<Source>
}
#[derive(PartialEq, Debug, Clone)]
pub struct NamedType {
    pub kind: Kinds,
    pub value: Option<String>,
    pub loc: Option<Location>
}
#[derive(PartialEq, Debug)]
pub struct Argument {
    pub kind: Kinds,
    pub name: NamedType,
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
    VariableValue { kind: Kinds, name: NamedType, loc: Option<Location> },
}

#[derive(PartialEq, Debug)]
pub struct ObjectField {
    pub kind: Kinds,
    pub name: NamedType,
    pub value: Value,
    pub loc: Option<Location>
}
