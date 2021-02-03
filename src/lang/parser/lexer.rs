use super::token::{Token, TokenTag};
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{BufRead, BufReader, Lines, Read};

type TokenMatcher = Box<dyn Fn(&str) -> usize + Sync>;
type TokenGenerator = Box<dyn Fn(&str) -> Option<Token> + Sync>;
type LexerRule = (TokenMatcher, TokenGenerator);

static KW_RULES: [(&'static str, TokenTag); 23] = [
    ("bool", TokenTag::KwBool),
    ("i32", TokenTag::KwI32),
    ("f64", TokenTag::KwF64),
    ("class", TokenTag::KwClass),
    ("self", TokenTag::KwLSelf),
    ("new", TokenTag::KwNew),
    ("super", TokenTag::KwSuper),
    ("crate", TokenTag::KwCrate),
    ("static", TokenTag::KwStatic),
    ("let", TokenTag::KwLet),
    ("fn", TokenTag::KwLFn),
    ("as", TokenTag::KwAs),
    ("for", TokenTag::KwFor),
    ("while", TokenTag::KwWhile),
    ("loop", TokenTag::KwLoop),
    ("if", TokenTag::KwIf),
    ("else", TokenTag::KwElse),
    ("continue", TokenTag::KwContinue),
    ("break", TokenTag::KwBreak),
    ("return", TokenTag::KwReturn),
    ("true", TokenTag::KwTrue),
    ("false", TokenTag::KwFalse),
    ("null", TokenTag::KwNull),
];

static OP_RULES: [(&'static str, TokenTag); 32] = [
    ("+", TokenTag::Add),
    ("-", TokenTag::Sub),
    ("*", TokenTag::Mul),
    ("/", TokenTag::Div),
    ("%", TokenTag::Mod),
    ("==", TokenTag::Eq),
    (">", TokenTag::Gt),
    (">=", TokenTag::Ge),
    ("<", TokenTag::Lt),
    ("<=", TokenTag::Le),
    ("!=", TokenTag::Ne),
    ("&&", TokenTag::LogAnd),
    ("||", TokenTag::LogOr),
    ("!", TokenTag::LogNot),
    ("=", TokenTag::Assign),
    ("+=", TokenTag::AddAssign),
    ("-=", TokenTag::SubAssign),
    ("*=", TokenTag::MulAssign),
    ("/=", TokenTag::DivAssign),
    ("%=", TokenTag::ModAssign),
    ("->", TokenTag::Arrow),
    ("::", TokenTag::DoubleColon),
    (":", TokenTag::Colon),
    (".", TokenTag::Dot),
    ("{", TokenTag::LBraces),
    ("}", TokenTag::RBraces),
    ("(", TokenTag::LParen),
    (")", TokenTag::RParen),
    ("[", TokenTag::LBracket),
    ("]", TokenTag::RBracket),
    (";", TokenTag::SemiColon),
    (",", TokenTag::Comma),
];

static MISC_RULES: [(&'static str, fn(&str) -> Option<Token>); 7] = [
    (
        // Identifier
        r"^[_a-zA-Z][_a-zA-Z0-9]*",
        |s: &str| Some(Token::new(TokenTag::Id, Some(String::from(s)))),
    ),
    (
        // Comment
        r"^//.*",
        |_s: &str| None,
    ),
    (
        // White Space
        r"^\s+",
        |_s: &str| None,
    ),
    (
        // Decimal number
        r"^\d+",
        |s: &str| Some(Token::new(TokenTag::DecLiteral, Some(String::from(s)))),
    ),
    (
        // Float number
        r"^\d+\.\d+",
        |s: &str| Some(Token::new(TokenTag::FpLiteral, Some(String::from(s)))),
    ),
    (
        // String literal
        r#"^"[^\\"]*(\\.[^\\"]*)*""#,
        |s: &str| Some(Token::new(TokenTag::StrLiteral, Some(String::from(s)))),
    ),
    (
        // Char literal
        r"^'[^\\']*(\\.[^\\']*)*'",
        |s: &str| Some(Token::new(TokenTag::ChLiteral, Some(String::from(s)))),
    ),
];

lazy_static! {
    static ref LEXICAL_RULES: Vec<LexerRule> = {
    let mut rules: Vec<LexerRule> = Vec::new();
    for rule in KW_RULES.iter() {
        rules.push((
            Box::new(move |s: &str| -> usize {
                if s.starts_with(rule.0) {
                    return rule.0.len();
                }
                0 // Return 0 indicates no match
            }),
            Box::new(move |_s: &str| -> Option<Token> { Some(Token::new(rule.1, None)) }),
        ));
    }

    for rule in OP_RULES.iter() {
        rules.push((
            Box::new(move |s: &str| -> usize {
                if s.starts_with(rule.0) {
                    return rule.0.len();
                }
                0 // Return 0 indicates no match
            }),
            Box::new(move |_s: &str| -> Option<Token> { Some(Token::new(rule.1, None)) }),
        ));
    }

    for rule in MISC_RULES.iter() {
        let regex_rule = Regex::new(rule.0).unwrap();
        rules.push((
            Box::new(move |s: &str| -> usize {
                let m = regex_rule.find(s);
                if m.is_some() {
                    return m.unwrap().end();
                }
                0 // Return 0 indicates no match
            }),
            Box::new(rule.1),
        ));
    }
    rules
    };
}

pub struct Lexer<F> {
    _lines: Lines<BufReader<F>>,
    _line: String,
    _row: usize,
    _col: usize,
}

impl<F: Read> Lexer<F> {
    pub fn new(s: F) -> Lexer<F> {
        let lines = BufReader::new(s).lines();
        Lexer {
            _lines: lines,
            _line: String::new(),
            _row: 0,
            _col: 0,
        }
    }
}

impl<'rule, F: Read> Iterator for Lexer<F> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self._col >= self._line.len() {
                match self._lines.next() {
                    Some(l) => {
                        self._line = l.unwrap();
                    }
                    None => {
                        // EOF reached
                        return None;
                    }
                }
                self._col = 0;
                self._row += 1;
                // This line might be empty, check at next iteration
                continue;
            }

            let mut match_len: usize = 0;
            let mut generator: Option<&TokenGenerator> = None;
            for rule in LEXICAL_RULES.iter() {
                // println!("{}", &self._line[self._col..]);
                let len = rule.0(&self._line[self._col..]);
                if len > match_len {
                    match_len = len;
                    generator = Some(&rule.1);
                }
            }

            if match_len == 0 {
                // No match
                panic!("Lexer: No rule for match");
            }

            let t = generator.unwrap()(&self._line[self._col..self._col + match_len]);
            self._col += match_len;
            if t.is_some() {
                break Some(t.unwrap());
            }
        }
    }
}
