#[macro_use]
extern crate log;
extern crate env_logger;

pub mod language;
pub mod types;
pub mod executor;

use types::definition::Schema;
use language::lexer::Source;
use language::parser::{Parser, ParseOptions};
use executor::Executor;

pub struct GraphQL;

impl GraphQL {
    pub fn query(schema: Schema, query: String) -> String {
        let source = Source::from(query);
        let ast    = Parser::parse(source, ParseOptions::new());
        if ast.is_ok() {
            // TODO validate
            let _ = Executor::execute(schema, ast.ok().unwrap());
            String::from("Foo")
        } else {
            panic!("FUCK!");
        }
    }
}
