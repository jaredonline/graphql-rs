#[derive(PartialEq, Debug, Clone)]
pub struct Source {
    body: String,
    name: Option<String>
}

impl Source {
    pub fn new(body: &str) -> Source {
        Source::from(String::from(body))
    }

    pub fn from(body: String) -> Source {
        Source {
            body: body,
            name: None
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenKind {
    EOF,
    Bang,
    Dollar,
    ParenL,
    ParenR,
    Spread,
    Colon,
    Equals,
    At,
    BracketL,
    BracketR,
    BraceR,
    BraceL,
    Pipe,
    Name,
    Variable,
    Int,
    Float,
    String
}

#[derive(PartialEq, Debug)]
pub enum NameKind {
    Query,
    Mutation,
    Fragment
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
    pub value: Option<String>
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

    pub fn name_kind_from_value(&self) -> Option<NameKind> {
        match self.value {
            Some(ref v) => {
                if *v == "mutation".to_string() {
                    Some(NameKind::Mutation)
                } else if *v == "query".to_string() {
                    Some(NameKind::Query)
                } else if *v == "fragment".to_string() {
                    Some(NameKind::Fragment)
                } else {
                    None
                }
            },
            None => None
        }
    }
}

pub struct Lexer {
    prev_position: usize,
    source: Source,
}

impl Lexer {
    pub fn lex(source: Source) -> Lexer {
        Lexer {
            prev_position: 0,
            source: source,
        }
    }

    pub fn next(&mut self, reset_position: Option<usize>) -> Token {
        let token = match reset_position {
            Some(i) => Lexer::read_token(&self.source, i),
            None    => Lexer::read_token(&self.source, self.prev_position)
        };

        self.prev_position = token.end;
        token
    }

    fn read_token(source: &Source, from_position: usize) -> Token {
        let ref body = source.body;

        let position = Lexer::position_after_whitespace(body, from_position);
        let mut bytes = body.bytes();
        let c = bytes.nth(position);
        match c {
            Some(code) => {
                match code {
                    // !
                    33 => Token::make_char(TokenKind::Bang, position),
                    // $
                    36 => Token::make_char(TokenKind::Dollar, position),
                    // (
                    40 => Token::make_char(TokenKind::ParenL, position),
                    // )
                    41 => Token::make_char(TokenKind::ParenR, position),
                    // .
                    46 => {
                        // test for ...
                        let c1 = bytes.next().unwrap_or(0);
                        let c2 = bytes.next().unwrap_or(0);
                        if c1 == 46 && c2 == 46 {
                            Token::make(TokenKind::Spread, position, position + 3)
                        } else {
                            // TODO throw error
                            Token::make(TokenKind::EOF, position, position)
                        }
                    }
                    // :
                    58 => Token::make_char(TokenKind::Colon, position),
                    // =
                    61 => Token::make_char(TokenKind::Equals, position),
                    // @
                    64 => Token::make_char(TokenKind::At, position),
                    // [
                    91 => Token::make_char(TokenKind::BracketL, position),
                    // ]
                    93 => Token::make_char(TokenKind::BracketR, position),
                    // {
                    123 => Token::make_char(TokenKind::BraceL, position),
                    // |
                    124 => Token::make_char(TokenKind::Pipe, position),
                    // }
                    125 => Token::make_char(TokenKind::BraceR, position),
                    // A-Z _ a-z
                    65 ... 90 | 95 | 97 ... 122 => Lexer::read_name(source, position),

                    // 0-9
                    45 | 48 ... 57 => Lexer::read_number(source, position),

                    // "
                    34 => Lexer::read_string(source, position),

                    // TODO throw error
                    _ => Token::make(TokenKind::EOF, position, position)
                }
            },
            _ => Token::make(TokenKind::EOF, position, position)
        }
    }

    fn read_string(source: &Source, start: usize) -> Token {
        let ref body = source.body;
        let mut position = start + 1;
        let mut chunk_start = position;
        let mut bytes = body.bytes();
        let mut code = bytes.nth(position).unwrap();
        let mut value = vec![];

        while 
            position < body.len() &&
            code != 34 &&
            code != 10 && code != 13 // TODO && code != 0x2028 && code != 0x2029
        {
            position += 1;
            if code == 92 { // \
                Lexer::push_bytes_to_byte_array(body, &mut value, chunk_start, position - 1);
                code = bytes.next().unwrap();
                match code {
                    //34 | 47 | 92 | 98 | 102 | 110 | 114 | 116 => value.push(code),

                    _ => { } // TODO throw
                }
                position += 1;
                chunk_start = position;
            }
            code = bytes.next().unwrap_or(0);
        }

        if code != 34 {
            // TODO throw
        }

        Lexer::push_bytes_to_byte_array(body, &mut value, chunk_start, position);
        Token {
            kind: TokenKind::String,
            start: start,
            end: position + 1,
            value: Some(String::from_utf8(value).unwrap())
        }
    }

    fn push_bytes_to_byte_array(body: &String, target: &mut Vec<u8>, start: usize, end: usize) {
        let mut bytes = body.bytes();
        let mut i = start;
        target.push(bytes.nth(start).unwrap());
        while i < end - 1 {
            i += 1;
            target.push(bytes.next().unwrap());
        }
    }

    fn read_name(source: &Source, position: usize) -> Token {
        let ref body = source.body;
        let mut bytes = body.bytes();
        let body_len = body.len();
        let mut end = position + 1;
        let mut code = bytes.nth(end).unwrap();
        while 
            end != body_len &&
            (
                code == 95 || // _
                (code >= 48 && code <= 57) || // 0-9
                (code >= 65 && code <= 90) || // A-Z
                (code >= 97 && code <= 122) // a-z
            )
        {
            end += 1;
            code = bytes.next().unwrap()
        }

        let string = Lexer::substring_from_body(body, position, end);

        Token {
            kind:  TokenKind::Name,
            start: position,
            end:   end,
            value: Some(string)
        }
    }

    fn substring_from_body(body: &String, start: usize, end: usize) -> String {
        // TODO: Figure out a better way to get a simple substring
        let mut bytes = body.bytes();
        let mut vec = vec![];
        let mut i = start;
        vec.push(bytes.nth(i).unwrap());
        while i < end - 1 {
            vec.push(bytes.next().unwrap());
            i += 1;
        }
        String::from_utf8(vec).unwrap()
    }

    fn read_number(source: &Source, start: usize) -> Token {
        let ref body = source.body;
        let mut bytes = body.bytes();
        let mut code = bytes.nth(start).unwrap();
        let mut position = start;
        let mut is_float = false;

        if code == 45 { // -
            code = bytes.next().unwrap();
            position += 1;
        }

        if code == 48 { // 0
            code = bytes.next().unwrap_or(0);
            position += 1;
        } else if code >= 49 && code <= 57 { // 1 - 9
            while code >= 48 && code <= 57 { // 0 - 9
                code = bytes.next().unwrap_or(0);
                position += 1;
            }
        } else {
            // TODO: throw invalid number
        }

        if code == 46 { // .
            is_float = true;

            code = bytes.next().unwrap();
            position += 1;

            if code >= 48 && code <= 57 { // 0-9
                while code >= 48 && code <= 57 {
                    code = bytes.next().unwrap_or(0);
                    position += 1;
                }
            } else {
                // TODO throw invalid number
            }
        }

        if code == 69 || code == 101 { // e or E
            is_float = true;

            code = bytes.next().unwrap();
            position += 1;

            if code == 43 || code == 45 { // + -
                code = bytes.next().unwrap();
                position += 1;
            }

            if code >= 48 && code <= 57 { // 0-9
                while code >= 48 && code <= 57 {
                    code = bytes.next().unwrap_or(0);
                    position += 1;
                }
            } else {
                // TODO throw invalid number
            }
        }

        let kind = match is_float {
            true => TokenKind::Float,
            false => TokenKind::Int
        };

        Token {
            kind: kind,
            start: start,
            end: position,
            value: Some(Lexer::substring_from_body(body, start, position))
        }
    }

    fn position_after_whitespace(body: &String, start_position: usize) -> usize {
        let body_length = body.len();
        let mut position = start_position;
        let mut bytes = body.bytes();

        let mut c = bytes.nth(position);
        while position < body_length {
            match c {
                Some(mut code) => {
                    if Lexer::is_whitespace(code) {
                        position += 1;
                    } else if code == 35 { // skip comments
                        position += 1;
                        while position < body_length {
                            code = bytes.next().unwrap();
                            position += 1;
                            if code == 10 || code == 13 { // TODO || code == 0x2028 || code == 0x2029 {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                },
                None => {
                    break
                }
            }
            c = bytes.next();
        }
        position
    }

    fn is_whitespace(code: u8) -> bool {
        code == 32 || code == 44 || code == 160 || // TODO:  code == 0x2028 || code == 0x2029 ||
        (code > 8 && code < 14)
    }
}

#[cfg(test)]
mod test {
    use env_logger;
    use super::*;

    fn lex_one(body: &str) -> Token {
        Lexer::lex(Source::new(body)).next(None)
    }

    #[test]
    fn it_skips_whitespace() {
        //let _ = env_logger::init();
        assert_eq!(lex_one("

        foo

        "),
        Token {
            kind: TokenKind::Name,
            start: 10,
            end: 13,
            value: Some("foo".to_string())
        });

        assert_eq!(lex_one("
        #comment
        foo#comment
        "),
        Token {
            kind: TokenKind::Name,
            start: 26,
            end: 29,
            value: Some("foo".to_string())
        });

        assert_eq!(lex_one(",,,foo,,,,"),
        Token {
            kind: TokenKind::Name,
            start: 3,
            end: 6,
            value: Some("foo".to_string())
        });
    }

    #[test]
    fn it_lexes_numbers() {
        //let _ = env_logger::init();

        assert_eq!(lex_one("4"), Token {
            kind: TokenKind::Int,
            start: 0,
            end: 1,
            value: Some("4".to_string())
        });

        assert_eq!(lex_one("4.123"), Token {
            kind: TokenKind::Float,
            start: 0,
            end: 5,
            value: Some("4.123".to_string())
        });

        assert_eq!(lex_one("-4"), Token {
            kind: TokenKind::Int,
            start: 0,
            end: 2,
            value: Some("-4".to_string())
        });

        assert_eq!(lex_one("9"), Token {
            kind: TokenKind::Int,
            start: 0,
            end: 1,
            value: Some("9".to_string())
        });

        assert_eq!(lex_one("0"), Token {
            kind: TokenKind::Int,
            start: 0,
            end: 1,
            value: Some("0".to_string())
        });

        assert_eq!(lex_one("00"), Token {
            kind: TokenKind::Int,
            start: 0,
            end: 1,
            value: Some("0".to_string())
        });

        assert_eq!(lex_one("-4.123"), Token {
            kind: TokenKind::Float,
            start: 0,
            end: 6,
            value: Some("-4.123".to_string())
        });

        assert_eq!(lex_one("0.123"), Token {
            kind: TokenKind::Float,
            start: 0,
            end: 5,
            value: Some("0.123".to_string())
        });

        assert_eq!(lex_one("123e4"), Token {
            kind: TokenKind::Float,
            start: 0,
            end: 5,
            value: Some("123e4".to_string())
        });

        assert_eq!(lex_one("123E4"), Token {
            kind: TokenKind::Float,
            start: 0,
            end: 5,
            value: Some("123E4".to_string())
        });

        assert_eq!(lex_one("123e-4"), Token {
            kind: TokenKind::Float,
            start: 0,
            end: 6,
            value: Some("123e-4".to_string())
        });

        assert_eq!(lex_one("123e+4"), Token {
            kind: TokenKind::Float,
            start: 0,
            end: 6,
            value: Some("123e+4".to_string())
        });

        assert_eq!(lex_one("-1.123e4"), Token {
            kind: TokenKind::Float,
            start: 0,
            end: 8,
            value: Some("-1.123e4".to_string())
        });

        assert_eq!(lex_one("-1.123E4"), Token {
            kind: TokenKind::Float,
            start: 0,
            end: 8,
            value: Some("-1.123E4".to_string())
        });

        assert_eq!(lex_one("-1.123e-4"), Token {
            kind: TokenKind::Float,
            start: 0,
            end: 9,
            value: Some("-1.123e-4".to_string())
        });

        assert_eq!(lex_one("-1.123e+4"), Token {
            kind: TokenKind::Float,
            start: 0,
            end: 9,
            value: Some("-1.123e+4".to_string())
        });

        assert_eq!(lex_one("-1.123e4567"), Token {
            kind: TokenKind::Float,
            start: 0,
            end: 11,
            value: Some("-1.123e4567".to_string())
        });

    }

    #[test]
    fn it_lexes_punctuation() {
        let _ = env_logger::init();

        fn test_punct(punc: &str, kind: TokenKind) {
            assert_eq!(lex_one(punc), Token {
                kind: kind,
                start: 0,
                end: 1,
                value: None
            });
        }

        test_punct("!", TokenKind::Bang);
        test_punct("$", TokenKind::Dollar);
        test_punct("(", TokenKind::ParenL);
        test_punct(")", TokenKind::ParenR);
        assert_eq!(lex_one("..."), Token {
            kind: TokenKind::Spread,
            start: 0,
            end: 3,
            value: None
        });
        test_punct(":", TokenKind::Colon);
        test_punct("=", TokenKind::Equals);
        test_punct("@", TokenKind::At);
        test_punct("[", TokenKind::BracketL);
        test_punct("]", TokenKind::BracketR);
        test_punct("{", TokenKind::BraceL);
        test_punct("}", TokenKind::BraceR);
        test_punct("|", TokenKind::Pipe);
    }

    #[test]
    fn it_lexes_strings() {
        let _ = env_logger::init();

        assert_eq!(lex_one("\"simple\""), Token {
            kind: TokenKind::String,
            start: 0,
            end: 8,
            value: Some("simple".to_string())
        });

        assert_eq!(lex_one(r#"" white space ""#), Token {
            kind: TokenKind::String,
            start: 0,
            end: 15,
            value: Some(" white space ".to_string())
        });

        assert_eq!(lex_one(r#""\"""#), Token {
            kind: TokenKind::String,
            start: 0,
            end: 4,
            value: Some(r#"\""#.to_string())
        });

        assert_eq!(lex_one(r#""quote \"""#), Token {
            kind: TokenKind::String,
            start: 0,
            end: 10,
            value: Some(r#"quote ""#.to_string())
        });
        
        // TODO: A bunch more tests that are a pain in the ass
    }

    #[test]
    fn it_finishes() {
        let _ = env_logger::init();

        let source = Source::new("{{");
        let mut lexer = Lexer::lex(source);
        lexer.next(None);
        lexer.next(None);
        assert_eq!(lexer.next(None).kind, TokenKind::EOF);
    }

    // TODO: exception based tests
}
