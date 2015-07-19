extern crate env_logger;

struct Source {
    body: String,
    name: Option<String>
}

impl Source {
    fn new(body: &str) -> Source {
        Source {
            body: body.to_string(),
            name: None
        }
    }
}

#[derive(PartialEq, Debug)]
enum TokenKind {
    EOF,
    BANG,
    DOLLAR,
    PAREN_L,
    PAREN_R,
    SPREAD,
    COLON,
    EQUALS,
    AT,
    BRACKET_L,
    BRACKET_R,
    BRACE_R,
    BRACE_L,
    PIPE,
    NAME,
    VARIABLE,
    INT,
    FLOAT,
    STRING
}

#[derive(PartialEq, Debug)]
struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
    value: Option<String>
}

impl Token {
    fn make(kind: TokenKind, start: usize, end: usize) -> Token {
        Token {
            kind: kind,
            start: start,
            end: end,
            value: None
        }
    }

    fn make_char(kind: TokenKind, start: usize) -> Token {
        Token {
            kind: kind,
            start: start,
            end: start + 1,
            value: None
        }
    }
}

struct Lexer {
    prev_position: usize,
    source: Source,
}

impl Lexer {
    fn lex(source: Source) -> Lexer {
        Lexer {
            prev_position: 0,
            source: source,
        }
    }

    fn next(&mut self, reset_position: Option<usize>) -> Token {
        let token = match reset_position {
            Some(i) => Lexer::read_token(&self.source, i),
            None    => Lexer::read_token(&self.source, self.prev_position)
        };

        self.prev_position = token.end;
        token
    }

    fn read_token(source: &Source, from_position: usize) -> Token {
        let ref body = source.body;
        let body_length = body.len();

        let position = Lexer::position_after_whitespace(body, from_position);
        let mut bytes = body.bytes();
        let code = bytes.nth(position);

        match code {
            Some(c) => {
                match c {
                    // !
                    33 => Token::make_char(TokenKind::BANG, position),
                    // $
                    36 => Token::make_char(TokenKind::DOLLAR, position),
                    // (
                    40 => Token::make_char(TokenKind::PAREN_L, position),
                    // )
                    41 => Token::make_char(TokenKind::PAREN_R, position),
                    // .
                    46 => {
                        // test for ...
                        let c1 = bytes.nth(position + 1).unwrap_or(0);
                        let c2 = bytes.nth(position + 2).unwrap_or(0);
                        if (c1 == 46 && c2 == 46) {
                            Token::make(TokenKind::SPREAD, position, position + 3)
                        } else {
                            // TODO throw error
                            Token::make(TokenKind::EOF, position, position)
                        }
                    }
                    // :
                    58 => Token::make_char(TokenKind::COLON, position),
                    // =
                    61 => Token::make_char(TokenKind::EQUALS, position),
                    // @
                    64 => Token::make_char(TokenKind::AT, position),
                    // [
                    91 => Token::make_char(TokenKind::BRACKET_L, position),
                    // ]
                    93 => Token::make_char(TokenKind::BRACKET_R, position),
                    // {
                    123 => Token::make_char(TokenKind::BRACE_L, position),
                    // |
                    124 => Token::make_char(TokenKind::PIPE, position),
                    // }
                    125 => Token::make_char(TokenKind::BRACE_R, position),
                    // A-Z _ a-z
                    65 ... 90 | 95 | 97 ... 122 => Lexer::read_name(source, position),

                    // TODO throw error
                    _ => Token::make(TokenKind::EOF, position, position)
                }
            }
            None => Token::make(TokenKind::EOF, position, position)
        }
    }

    fn read_name(source: &Source, position: usize) -> Token {
        let ref body = source.body;
        let mut bytes = body.bytes();
        let body_len = body.len();
        let mut end = position + 1;
        let mut code = 0;
        code = bytes.nth(end).unwrap();
        while (
            end != body_len &&
            (
                code == 95 || // _
                (code >= 48 && code <= 57) || // 0-9
                (code >= 65 && code <= 90) || // A-Z
                (code >= 97 && code <= 122) // a-z
            )
        ) {
            end += 1;
            code = bytes.next().unwrap()
        }

        // TODO: Figure out a better way to get a simple substring
        let mut by = body.bytes();
        let mut vec = vec![];
        let mut i = position;
        vec.push(by.nth(i).unwrap());
        while (i < end - 1) {
            vec.push(by.next().unwrap());
            i += 1;
        }
        let string = String::from_utf8(vec).unwrap();

        Token {
            kind: TokenKind::NAME,
            start: position,
            end: end,
            value: Some(string)
        }
    }

    fn position_after_whitespace(body: &String, start_position: usize) -> usize {
        let body_length = body.len();
        let mut position = start_position;
        let mut bytes = body.bytes();

        let mut c = bytes.nth(position);
        while (position < body_length) {
            match c {
                Some(mut code) => {
                    if Lexer::is_whitespace(code) {
                        position += 1;
                    } else if code == 35 { // skip comments
                        position += 1;
                        while (position < body_length) {
                            code = bytes.next().unwrap();
                            position += 1;
                            if (code == 10 || code == 13 || code == 0x2028 || code == 0x2029) {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                },
                None => {
                    error!("did not unwrap char at {}!", position);
                    break
                }
            }
            c = bytes.next();
        }
        position
    }

    fn is_whitespace(code: u8) -> bool {
        code == 32 || code == 44 || code == 160 || code == 0x2028 || code == 0x2029 ||
        (code > 8 && code < 14)
    }
}

#[test]
fn it_skips_whitespace() {
    let _ = env_logger::init();
    let mut lexer = Lexer::lex(Source::new("

    foo

    "));

    let mut token = Token {
        kind: TokenKind::NAME,
        start: 6,
        end: 9,
        value: Some("foo".to_string())
    };

    assert_eq!(lexer.next(None), token);

    lexer = Lexer::lex(Source::new("
    #comment
    foo#comment
    "));

    token = Token {
        kind: TokenKind::NAME,
        start: 18,
        end: 21,
        value: Some("foo".to_string())
    };

    assert_eq!(lexer.next(None), token);

    lexer = Lexer::lex(Source::new(",,,foo,,,"));

    token = Token{
        kind: TokenKind::NAME,
        start: 3,
        end: 6,
        value: Some("foo".to_string())
    };

    assert_eq!(lexer.next(None), token);
}

// TODO: exception based tests
