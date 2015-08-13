#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Kinds {
    Document, 

    OperationDefinition,

    FragmentDefinition,

    SelectionSet,

    Field,

    Argument,

    Name,

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
