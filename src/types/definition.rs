use std::collections::HashMap;

pub struct Schema {
    pub query: Object
}

pub struct Enum {
    pub name: String,
    pub description: String,
    pub values: HashMap<String, EnumValue>
}

pub struct EnumValue {
    pub value: usize,
    pub description: String
}

pub struct Object {
    pub name: String
}
