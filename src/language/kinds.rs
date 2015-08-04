#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Kinds {
    Document, 

    OperationDefinition,

    FragmentDefinition,

    SelectionSet,

    Field,

    Name
}
