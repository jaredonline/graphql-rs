#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Kinds {
    Document, 

    OperationDefinition,

    FragmentDefinition,
    VariableDefinition,

    InlineFragment,
    FragmentSpread,

    SelectionSet,

    Field,
    Directive,

    Argument,

    Name,
    NonNullType,
    ListType,
    NamedType,

    Int,
    Float,
    String,
    Boolean,
    Enum,
    Array,
    Object,
    ObjectField,
    Variable
}
