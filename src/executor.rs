extern crate log;
extern crate env_logger;

use language::ast::Document;
use types::definition::Schema;

pub struct Executor;

impl Executor {
    pub fn execute(schema: Schema, document_ast: Document) {
    }
}
