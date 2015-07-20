use language::lexer::{Source, Lexer, Token, TokenKind};
use language::ast::Document;

pub struct ParseOptions {
    no_source: Option<bool>,
    no_location: Option<bool>
}

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
        let parser = Parser {
            lex_token: lexer,
            source: source,
            options: options,
            prev_end: 0,
            token: token
        };
        Parser::parse_document(parser)
    }

    fn parse_document(parser: Parser) -> Document {
        let mut parser = parser;
        let start = parser.token.start;
        //let definitions = vec![];

        // rust doesn't have do/while so we make our own
        let mut cont = true;
        while cont {
            if Parser::peek(&parser, TokenKind::BraceL) {
            } else if Parser::peek(&parser, TokenKind::NAME) {
            } else {
                // TODO throw
            }
            cont = !Parser::skip(&mut parser, TokenKind::EOF);
        }
        Document
    }

    fn skip(parser: &mut Parser, kind: TokenKind) -> bool {
        error!("skip if {:?}", parser.token.kind);
        match parser.token.kind == kind {
            true => {
                error!("found {:?}", kind);
                Parser::advance(parser);
                true
            }
            _ => false
        }
    }

    fn advance(parser: &mut Parser) {
        let prev_end = parser.token.end;
        parser.prev_end = prev_end;
        parser.token = parser.lex_token.next(None);
    }

    fn peek(parser: &Parser, kind: TokenKind) -> bool {
        parser.token.kind == kind
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use language::lexer::{Source};

    #[test]
    fn it_accepts_option_to_not_include_source() {
        let source = Source::new("{ field }");
        Parser::parse(source, ParseOptions { no_source: Some(true), no_location: None });
    }
}
