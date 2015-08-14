use language::lexer::{Source, Lexer, Token, TokenKind, NameKind};
use language::ast::{Document, Definition, Directive, SelectionSet, Location, Selection, Type, Argument, Value, ObjectField, VariableDefinition, Name};
use language::kinds::Kinds;

use std::sync::RwLock;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ParseOptions {
    no_source: Option<bool>,
    no_location: Option<bool>
}

type RwParser = RwLock<Parser>;

pub struct Parser {
    lex_token: Lexer,
    source: Source,
    options: ParseOptions,
    prev_end: usize,
    token: Token
}

impl Parser {
    pub fn parse(source: Source, options: ParseOptions) -> Document {
        let mut lexer = Lexer::lex(source.clone());
        let token = lexer.next(None);
        let parser = RwLock::new(Parser {
            lex_token: lexer,
            source: source,
            options: options,
            prev_end: 0,
            token: token
        });
        Parser::parse_document(&parser)
    }

    fn parse_document(parser: &RwParser) -> Document {
        let start = { parser.read().unwrap().token.start };
        let mut definitions = vec![];

        // rust doesn't have do/while so we make our own
        let mut cont = true;
        while cont {
            if Parser::peek(parser, TokenKind::BraceL) {
                definitions.push(Parser::parse_operation_definition(parser));
            } else if Parser::peek(parser, TokenKind::Name) {
                let name_kind = { parser.read().unwrap().token.name_kind_from_value() };
                match name_kind {
                    Some(s) => {
                        match s {
                            NameKind::Query | NameKind::Mutation => {
                                definitions.push(Parser::parse_operation_definition(parser));
                            },
                            NameKind::Fragment => {
                                definitions.push(Parser::parse_fragment_definition(parser));
                            }
                        };
                    },
                    None => { panic!("no name_kind?"); }
                };
            } else {
                panic!("couldn't peek?");
            }
            cont = !Parser::skip(parser, TokenKind::EOF);
        }
        Document {
            kind: Kinds::Document,
            loc: Parser::loc(parser, start),
            definitions: definitions
        }
    }

    fn parse_operation_definition(parser: &RwParser) -> Definition {
        let start = { parser.read().unwrap().token.start };
        if Parser::peek(parser, TokenKind::BraceL) {
            Definition::Operation {
                kind: Kinds::OperationDefinition,
                operation: "query".to_string(),
                name: None,
                variable_definitions: None,
                directives: vec![],
                selection_set: Parser::parse_selection_set(parser),
                loc: Parser::loc(parser, start)
            }
        } else {
            let operation_token = Parser::expect(parser, TokenKind::Name);
            let operation = operation_token.value;
            Definition::Operation {
                kind: Kinds::OperationDefinition,
                operation: operation.unwrap(),
                name: Some(Parser::parse_name(parser)),
                variable_definitions: Some(Parser::parse_variable_definitions(parser)),
                directives: Parser::parse_directives(parser),
                selection_set: Parser::parse_selection_set(parser),
                loc: Parser::loc(parser, start)
            }
        }
    }

    fn parse_variable_definitions(parser: &RwParser) -> Vec<VariableDefinition> {
        if Parser::peek(parser, TokenKind::ParenL) {
            Parser::many(parser, TokenKind::ParenL, |parser: &RwParser| -> VariableDefinition {
                let start = { parser.read().unwrap().token.start };
                VariableDefinition {
                    kind: Kinds::VariableDefinition,
                    variable: Parser::parse_variable(parser),
                    var_type: { Parser::expect(parser, TokenKind::Colon); Parser::parse_type(parser) },
                    default_value: match Parser::skip(parser, TokenKind::Equals) {
                        true => Some(Parser::parse_value(parser, true)),
                        false => None
                    },
                    loc: Parser::loc(parser, start)
                }
            }, TokenKind::ParenR)
        } else {
            vec![]
        }
    }

    fn parse_type(parser: &RwParser) -> Type {
        let start = { parser.read().unwrap().token.start };
        let mut _type;

        if Parser::skip(parser, TokenKind::BracketL) {
            let temp_type = Box::new(Parser::parse_type(parser));
            Parser::expect(parser, TokenKind::BracketR);
            _type = Type::List {
                kind: Kinds::ListType,
                t_type: temp_type,
                loc: Parser::loc(parser, start)
            }
        } else {
            _type = Parser::parse_named_type(parser);
        }

        if Parser::skip(parser, TokenKind::Bang) {
            return Type::NonNull {
                kind: Kinds::NonNullType,
                t_type: Box::new(_type),
                loc: Parser::loc(parser, start)
            };
        }

        return _type;
    }

    fn parse_named_type(parser: &RwParser) -> Type {
        let start = { parser.read().unwrap().token.start };
        Type::Named {
            kind: Kinds::NamedType,
            name: Parser::parse_name(parser),
            loc: Parser::loc(parser, start)
        }
    }

    fn parse_fragment_definition(parser: &RwParser) -> Definition {
        let start = { parser.read().unwrap().token.start };
        Parser::expect_keyword(parser, "fragment");
        Definition::Fragment {
            kind: Kinds::FragmentDefinition,
            name: Parser::parse_fragment_name(parser),
            type_condition: { Parser::expect_keyword(parser, "on"); Parser::parse_named_type(parser) },
            directives: Some(Parser::parse_directives(parser)),
            selection_set: Parser::parse_selection_set(parser),
            loc: Parser::loc(parser, start)
        }
    }

    fn parse_selection_set(parser: &RwParser) -> SelectionSet {
        let start = { parser.read().unwrap().token.start };
        SelectionSet {
            kind: Kinds::SelectionSet,
            selections: Parser::many(parser, TokenKind::BraceL, |parser: &RwParser| -> Selection {
                if Parser::peek(parser, TokenKind::Spread) {
                    Parser::parse_fragment(parser)
                } else {
                    Parser::parse_field(parser)
                }
            }, TokenKind::BraceR),
            loc: Parser::loc(parser, start)
        }
    }

    fn parse_fragment(parser: &RwParser) -> Selection {
        let start = { parser.read().unwrap().token.start };
        Parser::expect(parser, TokenKind::Spread);
        let value = { parser.read().unwrap().token.clone().value.unwrap_or("".to_string()) };
        if value == "on".to_string() {
            Parser::advance(parser);
            Selection::InlineFragment {
                kind: Kinds::InlineFragment,
                type_condition: Parser::parse_named_type(parser),
                directives: Some(Parser::parse_directives(parser)),
                selection_set: Parser::parse_selection_set(parser),
                loc: Parser::loc(parser, start)
            }
        } else {
            Selection::FragmentSpread {
                kind: Kinds::FragmentSpread,
                name: Parser::parse_fragment_name(parser),
                directives: Some(Parser::parse_directives(parser)),
                loc: Parser::loc(parser, start),
            }
        }
    }
    
    fn parse_fragment_name(parser: &RwParser) -> Name {
        let value = { parser.read().unwrap().token.clone().value.unwrap_or("".to_string()) };
        if value == "on".to_string() {
            panic!("not supposed ot be 'on'");
        }
        Parser::parse_name(parser)
    }

    fn parse_field(parser: &RwParser) -> Selection {
        let start = { parser.read().unwrap().token.start };
        let name_or_alias = Parser::parse_name(parser);
        let mut alias;
        let mut name;
        if Parser::skip(parser, TokenKind::Colon) {
            alias = Some(name_or_alias);
            name  = Parser::parse_name(parser);
        } else {
            alias = None;
            name  = name_or_alias;
        }

        Selection::Field {
            kind: Kinds::Field,
            alias: alias,
            name: name,
            arguments: Parser::parse_arguments(parser),
            directives: Parser::parse_directives(parser),
            selection_set: if Parser::peek(parser, TokenKind::BraceL) {
                Some(Parser::parse_selection_set(parser))
            } else {
                None
            },
            loc: Parser::loc(parser, start)
        }
    }

    fn parse_arguments(parser: &RwParser) -> Vec<Argument> {
        if Parser::peek(parser, TokenKind::ParenL) {
            Parser::many(parser, TokenKind::ParenL, |parser: &RwParser| -> Argument {
                let start = { parser.read().unwrap().token.start };
                Argument {
                    kind: Kinds::Argument,
                    name: Parser::parse_name(parser),
                    value: { Parser::expect(parser, TokenKind::Colon); Parser::parse_value(parser, false) },
                    loc: Parser::loc(parser, start)
                }
            }, TokenKind::ParenR)
        } else {
            vec![]
        }
    }

    fn parse_value(parser: &RwParser, is_const: bool) -> Value {
        let token = { parser.read().unwrap().token.clone() };
        let value = token.value;
        match token.kind {
            TokenKind::BracketL => Parser::parse_array(parser, is_const),
            TokenKind::BraceL   => Parser::parse_object(parser, is_const),
            TokenKind::Int      => {
                Parser::advance(parser);
                Value::IntValue {
                    kind: Kinds::Int,
                    value: value.clone().unwrap(),
                    loc: Parser::loc(parser, token.start)
                }
            },
            TokenKind::Float => {
                Parser::advance(parser);
                Value::FloatValue {
                    kind: Kinds::Float,
                    value: value.clone().unwrap(),
                    loc: Parser::loc(parser, token.start)
                }
            },
            TokenKind::String => {
                Parser::advance(parser);
                Value::StringValue {
                    kind: Kinds::String,
                    value: value.clone().unwrap(),
                    loc: Parser::loc(parser, token.start)
                }
            },
            TokenKind::Name  => {
                if value.clone().unwrap_or("".to_string()) == "true".to_string() || value.clone().unwrap_or("".to_string()) == "false".to_string() {
                    Parser::advance(parser);
                    return Value::BooleanValue {
                        kind: Kinds::Boolean,
                        value: value.clone().unwrap() == "true".to_string(),
                        loc: Parser::loc(parser, token.start)
                    };
                } else if value.clone().unwrap_or("".to_string()) != "null".to_string() {
                    Parser::advance(parser);
                    return Value::EnumValue {
                        kind: Kinds::Enum,
                        value: value.clone().unwrap(),
                        loc: Parser::loc(parser, token.start)
                    };
                }
                panic!("no value?");
            },
            TokenKind::Dollar => {
                match is_const {
                    true  => panic!("no value?"),
                    false => Parser::parse_variable(parser)
                }
            }
            _ => panic!("Unexpected token kind: {:?}", token.kind)
        }
    }

    fn parse_array(parser: &RwParser, is_const: bool) -> Value {
        let start = { parser.read().unwrap().token.start };
        Value::ArrayValue {
            kind: Kinds::Array,
            values: Parser::any(parser, TokenKind::BracketL, |parser: &RwParser| -> Value {
                return Parser::parse_value(parser, is_const);
            }, TokenKind::BracketR),
            loc: Parser::loc(parser, start),
        }
    }

    fn parse_object(parser: &RwParser, is_const: bool) -> Value {
        let start = { parser.read().unwrap().token.start };
        Parser::expect(parser, TokenKind::BraceL);
        let mut field_names : HashMap<String, bool> = HashMap::new();
        let mut fields = vec![];
        while !Parser::skip(parser, TokenKind::BraceR) {
            fields.push(Parser::parse_object_field(parser, is_const, &mut field_names));
        }
        Value::ObjectValue {
            kind: Kinds::Object,
            fields: fields,
            loc: Parser::loc(parser, start)
        }
    }

    fn parse_object_field(parser: &RwParser, is_const: bool, field_names: &mut HashMap<String, bool>) -> ObjectField {
        let start = { parser.read().unwrap().token.start };
        let parsed_name = Parser::parse_name(parser);
        let value = parsed_name.value.clone();
        if field_names.contains_key(&value) {
            panic!("Duplicate input object field {:?}", value);
        }
        field_names.insert(value, true);
        ObjectField {
            kind: Kinds::ObjectField,
            name: parsed_name,
            value: { Parser::expect(parser, TokenKind::Colon); Parser::parse_value(parser, is_const) },
            loc: Parser::loc(parser, start)
        }
    }

    fn parse_variable(parser: &RwParser) -> Value {
        let start = { parser.read().unwrap().token.start };
        Parser::expect(parser, TokenKind::Dollar);
        Value::VariableValue {
            kind: Kinds::Variable,
            name: Parser::parse_name(parser),
            loc: Parser::loc(parser, start)
        }
    }

    fn parse_name(parser: &RwParser) -> Name {
        let token = Parser::expect(parser, TokenKind::Name);
        Name {
            kind: Kinds::Name,
            value: token.value.unwrap(),
            loc: Parser::loc(parser, token.start)
        }
    }

    fn parse_directives(parser: &RwParser) -> Vec<Directive> {
        let mut directives = vec![];
        while Parser::peek(parser, TokenKind::At) {
            directives.push(Parser::parse_directive(parser));
        }
        return directives;
    }

    fn parse_directive(parser: &RwParser) -> Directive {
        let start = { parser.read().unwrap().token.start };
        Parser::expect(parser, TokenKind::At);
        Directive {
            kind: Kinds::Directive,
            name: Parser::parse_name(parser),
            arguments: Some(Parser::parse_arguments(parser)),
            loc: Parser::loc(parser, start)
        }
    }

    fn any<T, F>(parser: &RwParser, open_kind: TokenKind, parse_fn: F, close_kind: TokenKind) -> Vec<T>
        where F : Fn(&RwParser) -> T {
        Parser::expect(parser, open_kind);
        let mut nodes = vec![];

        while !Parser::skip(parser, close_kind) {
            nodes.push(parse_fn(parser));
        }

        return nodes;
    }

    fn many<T, F>(parser: &RwParser, open_kind: TokenKind, parse_fn: F, close_kind: TokenKind) -> Vec<T>
        where F : Fn(&RwParser) -> T {
        Parser::expect(parser, open_kind);
        let mut nodes = vec![parse_fn(parser)];

        while !Parser::skip(parser, close_kind) {
            nodes.push(parse_fn(parser));
        }

        return nodes;
    }

    fn expect_keyword(parser: &RwParser, keyword: &str) -> Token { 
        let token = { parser.read().unwrap().token.clone() };
        let value = token.value.clone().unwrap_or("".to_string());
        if token.kind == TokenKind::Name && value == keyword.to_string() {
            Parser::advance(parser);
            return token;
        }

        panic!("Expected {:?} and got 'FILL THIS IN'", keyword);
    }

    fn expect(parser: &RwParser, kind: TokenKind) -> Token {
        let token = { parser.read().unwrap().token.clone() };
        if token.kind == kind {
            Parser::advance(parser);
            return token;
        }

        panic!("Expected {:?}, found {:?}", kind, token.kind);
    }

    fn loc(parser: &RwParser, start: usize) -> Option<Location> {
        let options = { parser.read().unwrap().options.clone() };
        if options.no_location.unwrap_or(false) {
            None
        } else if options.no_source.unwrap_or(false) {
            Some(Location {
                start: start,
                end: { parser.read().unwrap().prev_end },
                source: None
            })
        } else {
            Some(Location {
                start: start,
                end: { parser.read().unwrap().prev_end },
                source: Some({ parser.read().unwrap().source.clone() })
            })
        }
    }

    fn skip(parser: &RwParser, kind: TokenKind) -> bool {
        let token_kind = { parser.read().unwrap().token.kind };
        match token_kind == kind {
            true => {
                Parser::advance(parser);
                true
            }
            _ => false
        }
    }

    fn advance(parser: &RwParser) {
        let mut parser = parser.write().unwrap();
        let prev_end = parser.token.end;
        parser.prev_end = prev_end;
        parser.token = parser.lex_token.next(None);
    }

    fn peek(parser: &RwParser, kind: TokenKind) -> bool {
        parser.read().unwrap().token.kind == kind
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use language::lexer::*;
    use language::ast::*;
    use language::kinds::*;

    use std::io::Read;
    use std::fs::File;
    use env_logger;

    fn loc_builder(start: usize, end: usize, source: Option<Source>) -> Option<Location> {
        Some(Location { start: start, end: end, source: source })
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
        let document = Parser::parse(source, ParseOptions { no_source: Some(true), no_location: None });
        assert_eq!(goal, document);
    }

    #[test]
    fn it_parses_variable_inline_values() {
        let source = Source::new("{ field(complex: { a: { b: [ $var ] } }) }");
        Parser::parse(source, ParseOptions { no_source: None, no_location: None });
    }

    #[test]
    fn it_parses_the_kitchen_sink() {
        let _ = env_logger::init();
        let mut f = File::open("test_data/kitchen-sink.graphql").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();

        let source = Source::new(s.trim());
        Parser::parse(source, ParseOptions { no_source: None, no_location: None });
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

        let result = Parser::parse(source.clone(), ParseOptions { no_source: None, no_location: None });

        let goal = Document {
            kind: Kinds::Document,
            loc: loc_builder(1, 62, Some(source.clone())),
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

        assert_eq!(goal, result);
    }
}
