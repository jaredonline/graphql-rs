extern crate graphql;
extern crate log;
extern crate env_logger;

use graphql::language::parser::*;
use graphql::language::lexer::*;
use graphql::language::ast::*;
use graphql::language::kinds::*;

use self::env_logger::*;

use std::io::Read;
use std::fs::File;

fn loc_builder(start: usize, end: usize, source: Option<Source>) -> Option<Location> {
    Some(Location { start: start, end: end, source: source })
}

macro_rules! parse_no_source {
    ($src:expr) => {
        {
            let source = Source::new($src);
            Parser::parse(source, ParseOptions::no_source())
        }
    };
}

#[test]
fn it_provides_useful_errors() {
    let mut document;
    document = parse_no_source!("notanoperation Foo { field }");
    assert_eq!(true, document.is_err());
    assert_eq!("Could not parse document, missing NameKind at location 0", document.err().unwrap().description);

    document = parse_no_source!("
{ ...MissingOn }
fragment MissingOn Type
");
    assert_eq!(true, document.is_err());
    assert_eq!("Expected 'on' and got 'Type'", document.err().unwrap().description);

    // TODO finish these
}

#[test]
fn it_accepts_option_to_not_include_source() {
    let goal = Document {
        kind: Kinds::Document,
        loc: loc_builder(0, 9, None),
        definitions: vec![
            Definition::Operation {
                kind: Kinds::OperationDefinition,
                loc: loc_builder(0, 9, None),
                operation: "query".to_string(),
                name: None,
                variable_definitions: None,
                directives: vec![],
                selection_set: SelectionSet {
                    kind: Kinds::SelectionSet,
                    loc: loc_builder(0, 9, None),
                    selections: vec![
                        Selection::Field {
                            kind: Kinds::Field,
                            loc: loc_builder(2, 7, None),
                            alias: None,
                            name: Name {
                                kind: Kinds::Name,
                                loc: loc_builder(2, 7, None),
                                value: "field".to_string()
                            },
                            arguments: vec![],
                            directives: vec![],
                            selection_set: None
                        }
                    ]
                }
            }
        ]
    };

    let source = Source::new("{ field }");
    let document = Parser::parse(source, ParseOptions::no_source());
    assert_eq!(goal, document.ok().unwrap());
}

#[test]
fn it_parses_variable_inline_values() {
    let source = Source::new("{ field(complex: { a: { b: [ $var ] } }) }");
    Parser::parse(source, ParseOptions::new());
}

#[test]
fn it_parses_the_kitchen_sink() {
    let _ = env_logger::init();
    let mut f = File::open("tests/data/kitchen-sink.graphql").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();

    let source = Source::new(s.trim());
    Parser::parse(source, ParseOptions::new());
}

#[test]
fn it_parsers_creates_ast() {
    let source = Source::new("
{
    node(id: 4) {
        id,
        name
    }
}
");

    let result = Parser::parse(source.clone(), ParseOptions::new());

    let goal = Document {
        kind: Kinds::Document,
        loc: loc_builder(1, 54, Some(source.clone())),
        definitions: vec![
            Definition::Operation {
                kind: Kinds::OperationDefinition,
                loc: loc_builder(1, 53, Some(source.clone())),
                operation: "query".to_string(),
                name: None,
                variable_definitions: None,
                directives: vec![],
                selection_set: SelectionSet {
                    kind: Kinds::SelectionSet,
                    loc: loc_builder(1, 53, Some(source.clone())),
                    selections: vec![
                        Selection::Field {
                            kind: Kinds::Field,
                            loc: loc_builder(7, 51, Some(source.clone())),
                            alias: None,
                            name: Name {
                                kind: Kinds::Name,
                                loc: loc_builder(7, 11, Some(source.clone())),
                                value: "node".to_string()
                            },
                            arguments: vec![
                                Argument {
                                    kind: Kinds::Argument,
                                    loc: loc_builder(12, 17, Some(source.clone())),
                                    name: Name {
                                        kind: Kinds::Name,
                                        loc: loc_builder(12, 14, Some(source.clone())),
                                        value: "id".to_string()
                                    },
                                    value: Value::IntValue {
                                        kind: Kinds::Int,
                                        loc: loc_builder(16, 17, Some(source.clone())),
                                        value: "4".to_string()
                                    }
                                }
                            ],
                            directives: vec![],
                            selection_set: Some(SelectionSet {
                                kind: Kinds::SelectionSet,
                                loc: loc_builder(19, 51, Some(source.clone())),
                                selections: vec![
                                    Selection::Field {
                                        kind: Kinds::Field,
                                        loc: loc_builder(29, 31, Some(source.clone())),
                                        alias: None,
                                        name: Name {
                                            kind: Kinds::Name,
                                            loc: loc_builder(29, 31, Some(source.clone())),
                                            value: "id".to_string()
                                        },
                                        arguments: vec![],
                                        directives: vec![],
                                        selection_set: None,
                                    },
                                    Selection::Field {
                                        kind: Kinds::Field,
                                        loc: loc_builder(41, 45, Some(source.clone())),
                                        alias: None,
                                        name: Name {
                                            kind: Kinds::Name,
                                            loc: loc_builder(41, 45, Some(source.clone())),
                                            value: "name".to_string()
                                        },
                                        arguments: vec![],
                                        directives: vec![],
                                        selection_set: None,
                                    }
                                ],
                            })
                        }
                    ]
                }
            }
        ]
    };

    assert_eq!(goal, result.ok().unwrap());
}
