use language::lexer::{Source, Lexer, Token, TokenKind, NameKind};
use language::ast::{
    Document,
    Definition,
    Directive,
    SelectionSet,
    Location,
    Selection,
    Type,
    Argument,
    Value,
    ObjectField,
    VariableDefinition,
    Name
};
use language::kinds::Kinds;
use language::errors::{
    ParseError,
};

use std::sync::RwLock;
use std::collections::HashMap;
use std::result::Result;

#[derive(Clone, Copy)]
pub struct ParseOptions {
    no_source:   bool,
    no_location: bool
}

impl ParseOptions {
    pub fn new() -> ParseOptions {
        ParseOptions {
            no_source:   false,
            no_location: false
        }
    }

    pub fn no_source() -> ParseOptions {
        ParseOptions {
            no_source:   true,
            no_location: false
        }
    }

    pub fn no_location() -> ParseOptions {
        ParseOptions {
            no_source:   false,
            no_location: true
        }
    }

    pub fn set_location(&mut self, location: bool) {
        self.no_location = location;
    }

    pub fn set_source(&mut self, source: bool) {
        self.no_source = source;
    }
}

pub struct Parser {
    lex_token: Lexer,
    source:    Source,
    options:   ParseOptions,
    prev_end:  usize,
    token:     Token
}

impl Parser {
    pub fn parse(source: Source, options: ParseOptions) -> Result<Document, ParseError> {
        let mut lexer = Lexer::lex(source.clone());
        let token = lexer.next(None);
        let parser = RwLock::new(Parser {
            lex_token: lexer,
            source: source,
            options: options,
            prev_end: 0,
            token: token
        });
        InternalParser::parse(parser)
    }
}

type RwParser = RwLock<Parser>;

struct InternalParser {
    parser: RwParser
}

impl InternalParser {
    fn parse(parser: RwParser) -> Result<Document, ParseError> {
        let ip = InternalParser {
            parser: parser
        };
        ip.parse_document()
    }

    // Helpers

    fn start(&self) -> usize {
        self.parser.read().unwrap().token.start
    }

    fn token_kind(&self) -> TokenKind {
        self.parser.read().unwrap().token.kind
    }

    fn options(&self) -> ParseOptions {
        self.parser.read().unwrap().options
    }

    fn prev_end(&self) -> usize {
        self.parser.read().unwrap().prev_end
    }

    fn source_clone(&self) -> Source {
        self.parser.read().unwrap().source.clone()
    }

    fn token_clone(&self) -> Token {
        self.parser.read().unwrap().token.clone()
    }

    // Parsers

    fn parse_document(&self) -> Result<Document, ParseError> {
        let start = self.start();
        let mut definitions = vec![];

        // rust doesn't have do/while so we make our own
        let mut cont = true;
        while cont {
            let definition = self.parse_definition(start);
            let mut ok_definition;

            if definition.is_ok() {
                ok_definition = definition.ok().unwrap();
            } else {
                return Err(definition.err().unwrap());
            }
            definitions.push(ok_definition);
            cont = !self.skip(TokenKind::EOF);
        }

        Ok(Document {
            kind: Kinds::Document,
            loc: self.loc(start),
            definitions: definitions
        })
    }

    fn parse_definition(&self, start: usize) -> Result<Definition, ParseError> {
        if self.peek(TokenKind::BraceL) {
            self.parse_operation_definition()
        } else if self.peek(TokenKind::Name) {
            let name_kind = { self.parser.read().unwrap().token.name_kind_from_value() };
            match name_kind {
                Some(s) => {
                    match s {
                        NameKind::Query | NameKind::Mutation => { self.parse_operation_definition() },
                        NameKind::Fragment => { self.parse_fragment_definition() }
                    }
                },
                None => {
                    parse_error!("Could not parse document, missing NameKind at location {:?}", start)
                }
            }
        } else {
            parse_error!("Expected a BraceL or a Name at location {:?}", start)
        }
    }

    fn parse_operation_definition(&self) -> Result<Definition, ParseError> {
        let start = self.start();
        if self.peek(TokenKind::BraceL) {
            Ok(Definition::Operation {
                kind: Kinds::OperationDefinition,
                operation: "query".to_string(),
                name: None,
                variable_definitions: None,
                directives: vec![],
                selection_set: self.parse_selection_set(),
                loc: self.loc(start)
            })
        } else {
            match self.expect(TokenKind::Name) {
                Ok(ot) => {
                    let operation = ot.value;
                    Ok(Definition::Operation {
                        kind: Kinds::OperationDefinition,
                        operation: operation.unwrap(),
                        name: Some(self.parse_name()),
                        variable_definitions: Some(self.parse_variable_definitions()),
                        directives: self.parse_directives(),
                        selection_set: self.parse_selection_set(),
                        loc: self.loc(start)
                    })
                },
                Err(e) => Err(e)
            }
        }
    }

    fn parse_fragment_definition(&self) -> Result<Definition, ParseError> {
        let start = self.start();
        match self.expect_keyword("fragment") {
            Ok(_) => {
                let name = self.parse_fragment_name();
                let type_condition = match self.expect_keyword("on") {
                    Ok(_)  => Ok(self.parse_named_type()),
                    Err(e) => Err(e)
                };
                match type_condition {
                    Ok(tc) => Ok(Definition::Fragment {
                        kind: Kinds::FragmentDefinition,
                        name: name,
                        type_condition: tc,
                        directives: Some(self.parse_directives()),
                        selection_set: self.parse_selection_set(),
                        loc: self.loc(start)
                    }),
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }

    fn parse_selection_set(&self) -> SelectionSet {
        let start = self.start();
        SelectionSet {
            kind: Kinds::SelectionSet,
            selections: self.many(TokenKind::BraceL, || -> Selection {
                if self.peek(TokenKind::Spread) {
                    self.parse_fragment()
                } else {
                    self.parse_field()
                }
            }, TokenKind::BraceR),
            loc: self.loc(start)
        }
    }

    fn parse_fragment(&self) -> Selection {
        let start = self.start();
        let _ = self.expect(TokenKind::Spread);
        let token = self.token_clone();
        let value = token.clone().value.unwrap_or("".to_string());
        if value == "on".to_string() {
            self.advance();
            Selection::InlineFragment {
                kind: Kinds::InlineFragment,
                type_condition: self.parse_named_type(),
                directives: Some(self.parse_directives()),
                selection_set: self.parse_selection_set(),
                loc: self.loc(start)
            }
        } else {
            Selection::FragmentSpread {
                kind: Kinds::FragmentSpread,
                name: self.parse_fragment_name(),
                directives: Some(self.parse_directives()),
                loc: self.loc(start),
            }
        }
    }

    fn parse_field(&self) -> Selection {
        let start = self.start();
        let name_or_alias = self.parse_name();
        let mut alias;
        let mut name;
        if self.skip(TokenKind::Colon) {
            alias = Some(name_or_alias);
            name  = self.parse_name();
        } else {
            alias = None;
            name  = name_or_alias;
        }

        Selection::Field {
            kind: Kinds::Field,
            alias: alias,
            name: name,
            arguments: self.parse_arguments(),
            directives: self.parse_directives(),
            selection_set: if self.peek(TokenKind::BraceL) {
                Some(self.parse_selection_set())
            } else {
                None
            },
            loc: self.loc(start)
        }
    }

    fn parse_name(&self) -> Name {
        match self.expect(TokenKind::Name) {
            Ok(token) => {
                Name {
                    kind: Kinds::Name,
                    value: token.value.unwrap(),
                    loc: self.loc(token.start)
                }
            },
            _ => { panic!("expected Name, found... something else") }
        }
    }

    fn parse_arguments(&self) -> Vec<Argument> {
        if self.peek(TokenKind::ParenL) {
            self.many(TokenKind::ParenL, || -> Argument {
                let start = self.start();
                Argument {
                    kind: Kinds::Argument,
                    name: self.parse_name(),
                    value: { let _ = self.expect(TokenKind::Colon); self.parse_value(false) },
                    loc: self.loc(start)
                }
            }, TokenKind::ParenR)
        } else {
            vec![]
        }
    }

    fn parse_variable_definitions(&self) -> Vec<VariableDefinition> {
        if self.peek(TokenKind::ParenL) {
            self.many(TokenKind::ParenL, || -> VariableDefinition {
                let start = self.start();
                VariableDefinition {
                    kind: Kinds::VariableDefinition,
                    variable: self.parse_variable(),
                    var_type: { let _ = self.expect(TokenKind::Colon); self.parse_type() },
                    default_value: match self.skip(TokenKind::Equals) {
                        true => Some(self.parse_value(true)),
                        false => None
                    },
                    loc: self.loc(start)
                }
            }, TokenKind::ParenR)
        } else {
            vec![]
        }
    }

    fn parse_directives(&self) -> Vec<Directive> {
        let mut directives = vec![];
        while self.peek(TokenKind::At) {
            directives.push(self.parse_directive());
        }
        return directives;
    }

    fn parse_fragment_name(&self) -> Name {
        let token = self.token_clone();
        let value = token.clone().value.unwrap_or("".to_string());
        if value == "on".to_string() {
            panic!("not supposed ot be 'on'");
        }
        self.parse_name()
    }

    fn parse_named_type(&self) -> Type {
        let start = self.start();
        Type::Named {
            kind: Kinds::NamedType,
            name: self.parse_name(),
            loc: self.loc(start)
        }
    }

    fn parse_value(&self, is_const: bool) -> Value {
        let token = self.token_clone();
        let value = token.value;
        match token.kind {
            TokenKind::BracketL => self.parse_array(is_const),
            TokenKind::BraceL   => self.parse_object(is_const),
            TokenKind::Int      => {
                self.advance();
                Value::IntValue {
                    kind: Kinds::Int,
                    value: value.clone().unwrap(),
                    loc: self.loc(token.start)
                }
            },
            TokenKind::Float => {
                self.advance();
                Value::FloatValue {
                    kind: Kinds::Float,
                    value: value.clone().unwrap(),
                    loc: self.loc(token.start)
                }
            },
            TokenKind::String => {
                self.advance();
                Value::StringValue {
                    kind: Kinds::String,
                    value: value.clone().unwrap(),
                    loc: self.loc(token.start)
                }
            },
            TokenKind::Name  => {
                if value.clone().unwrap_or("".to_string()) == "true".to_string() || value.clone().unwrap_or("".to_string()) == "false".to_string() {
                    self.advance();
                    return Value::BooleanValue {
                        kind: Kinds::Boolean,
                        value: value.clone().unwrap() == "true".to_string(),
                        loc: self.loc(token.start)
                    };
                } else if value.clone().unwrap_or("".to_string()) != "null".to_string() {
                    self.advance();
                    return Value::EnumValue {
                        kind: Kinds::Enum,
                        value: value.clone().unwrap(),
                        loc: self.loc(token.start)
                    };
                }
                panic!("no value?");
            },
            TokenKind::Dollar => {
                match is_const {
                    true  => panic!("no value?"),
                    false => self.parse_variable()
                }
            }
            _ => panic!("Unexpected token kind: {:?}", token.kind)
        }
    }

    fn parse_variable(&self) -> Value {
        let start = self.start();
        let _ = self.expect(TokenKind::Dollar);
        Value::VariableValue {
            kind: Kinds::Variable,
            name: self.parse_name(),
            loc: self.loc(start)
        }
    }

    fn parse_type(&self) -> Type {
        let start = self.start();
        let mut _type;

        if self.skip(TokenKind::BracketL) {
            let temp_type = Box::new(self.parse_type());
            let _ = self.expect(TokenKind::BracketR);
            _type = Type::List {
                kind: Kinds::ListType,
                t_type: temp_type,
                loc: self.loc(start)
            }
        } else {
            _type = self.parse_named_type();
        }

        if self.skip(TokenKind::Bang) {
            return Type::NonNull {
                kind: Kinds::NonNullType,
                t_type: Box::new(_type),
                loc: self.loc(start)
            };
        }

        return _type;
    }

    fn parse_directive(&self) -> Directive {
        let start = self.start();
        let _ = self.expect(TokenKind::At);
        Directive {
            kind: Kinds::Directive,
            name: self.parse_name(),
            arguments: Some(self.parse_arguments()),
            loc: self.loc(start)
        }
    }

    fn parse_array(&self, is_const: bool) -> Value {
        let start = self.start();
        Value::ArrayValue {
            kind: Kinds::Array,
            values: self.any(TokenKind::BracketL, || -> Value {
                self.parse_value(is_const)
            }, TokenKind::BracketR),
            loc: self.loc(start),
        }
    }

    fn parse_object(&self, is_const: bool) -> Value {
        let start = self.start();
        let _ = self.expect(TokenKind::BraceL);
        let mut field_names : HashMap<String, bool> = HashMap::new();
        let mut fields = vec![];
        while !self.skip(TokenKind::BraceR) {
            fields.push(self.parse_object_field(is_const, &mut field_names));
        }
        Value::ObjectValue {
            kind: Kinds::Object,
            fields: fields,
            loc: self.loc(start)
        }
    }

    fn parse_object_field(&self, is_const: bool, field_names: &mut HashMap<String, bool>) -> ObjectField {
        let start = self.start();
        let parsed_name = self.parse_name();
        let value = parsed_name.value.clone();
        if field_names.contains_key(&value) {
            panic!("Duplicate input object field {:?}", value);
        }
        field_names.insert(value, true);
        ObjectField {
            kind: Kinds::ObjectField,
            name: parsed_name,
            value: { let _ = self.expect(TokenKind::Colon); self.parse_value(is_const) },
            loc: self.loc(start)
        }
    }


    // Iteration
    fn many<T, F>(&self, open_kind: TokenKind, parse_fn: F, close_kind: TokenKind) -> Vec<T>
        where F : Fn() -> T {
        let _ = self.expect(open_kind);
        let mut nodes = vec![parse_fn()];

        while !self.skip(close_kind) {
            nodes.push(parse_fn());
        }

        return nodes;
    }

    fn any<T, F>(&self, open_kind: TokenKind, parse_fn: F, close_kind: TokenKind) -> Vec<T>
        where F : Fn() -> T {
        let _ = self.expect(open_kind);
        let mut nodes = vec![];

        while !self.skip(close_kind) {
            nodes.push(parse_fn());
        }

        return nodes;
    }

    // Introspection
    fn loc(&self, start: usize) -> Option<Location> {
        if self.options().no_location {
            None
        } else if self.options().no_source {
            Some(Location {
                start: start,
                end: self.prev_end(),
                source: None
            })
        } else {
            Some(Location {
                start: start,
                end: self.prev_end(),
                source: Some(self.source_clone())
            })
        }
    }
    
    fn peek(&self, kind: TokenKind) -> bool {
        self.parser.read().unwrap().token.kind == kind
    }

    fn skip(&self, kind: TokenKind) -> bool {
        match self.token_kind() == kind {
            true => {
                self.advance();
                true
            }
            _ => false
        }
    }

    fn expect_keyword(&self, keyword: &str) -> Result<Token, ParseError> { 
        let token = self.token_clone();
        let value = token.value.clone().unwrap_or("".to_string());
        if token.kind == TokenKind::Name && value == keyword.to_string() {
            self.advance();
            return Ok(token);
        }

        parse_error!("Expected '{}' and got '{}'", keyword, value)
    }

    fn expect(&self, kind: TokenKind) -> Result<Token, ParseError> {
        let token = self.token_clone();
        if token.kind == kind {
            self.advance();
            return Ok(token);
        }

        parse_error!("Expected {:?}, found {:?}", kind, token.kind)
    }

    fn advance(&self) {
        let mut parser = self.parser.write().unwrap();
        let prev_end = parser.token.end;
        parser.prev_end = prev_end;
        parser.token = parser.lex_token.next(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_parse_options() {
        let po = ParseOptions::new();
        assert_eq!(po.no_location, false);
        assert_eq!(po.no_source, false);
    }

    #[test]
    fn no_location_parse_options() {
        let po = ParseOptions::no_location();
        assert_eq!(po.no_location, true);
        assert_eq!(po.no_source, false);
    }

    #[test]
    fn no_source_parse_options() {
        let po = ParseOptions::no_source();
        assert_eq!(po.no_source, true);
        assert_eq!(po.no_location, false);
    }

    #[test]
    fn parse_options_setters() {
        let mut po = ParseOptions::new();
        po.set_location(true);
        po.set_source(true);
        assert_eq!(po.no_source, true);
        assert_eq!(po.no_location, true);
    }
}
