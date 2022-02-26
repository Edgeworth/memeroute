use std::str::FromStr;

use eyre::{eyre, Result};
use regex::Regex;

use crate::dsn::token::{Tok, Token};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lexer {
    data: Vec<char>,
    token: String,
    tokens: Vec<Token>,
    idx: usize,
    string_quote: Option<char>, // What the quote character is, out of ', ", $
    spaces_in_quotes: bool,     // If quoted strings can contain spaces.
}

impl Lexer {
    pub fn new(data: &str) -> Result<Self> {
        let string_quote_rx = Regex::new(r"(?is)\(\s*string_quote\s+(.)\s*\)")?;
        let spaces_in_quotes_rx = Regex::new(r"(?is)\(\s*space_in_quoted_tokens\s+on\s*\)")?;

        let string_quote = if let Some(cap) = string_quote_rx.captures(data) {
            let quote = cap.get(1).ok_or_else(|| eyre!("expected quote chacater"))?;
            match quote.as_str() {
                "'" | "\"" | "$" => quote.as_str().chars().next(),
                x => return Err(eyre!("unknown string quote character {}", x)),
            }
        } else {
            None
        };
        let spaces_in_quotes = spaces_in_quotes_rx.is_match(data);

        // Remove these directives. At least the string quote needs to
        // be removed for proper lexing.
        let data = string_quote_rx.replace_all(data, "");
        let data = spaces_in_quotes_rx.replace_all(&data, "");

        Ok(Self {
            data: data.chars().collect(),
            token: String::new(),
            tokens: Vec::new(),
            idx: 0,
            string_quote,
            spaces_in_quotes,
        })
    }

    pub fn lex(mut self) -> Result<Vec<Token>> {
        while self.idx < self.data.len() {
            let c = self.next()?;
            if Some(c) == self.string_quote {
                // Grab quoted literal.
                let stop = if self.spaces_in_quotes { self.string_quote.unwrap() } else { ' ' };
                while self.peek() != stop {
                    let next = self.next()?;
                    self.token.push(next);
                }
                self.next()?; // Discard ending character
                self.push();
            } else {
                // Ends current token:
                if c.is_whitespace() || c == '(' || c == ')' {
                    self.push();
                }
                if !c.is_whitespace() {
                    self.token.push(c);
                }
                // Is complete token:
                if c == '(' || c == ')' {
                    self.push();
                }
            }
        }
        self.push();
        Ok(self.tokens)
    }

    fn peek(&self) -> char {
        self.data[self.idx]
    }

    fn next(&mut self) -> Result<char> {
        if self.idx < self.data.len() {
            self.idx += 1;
            Ok(self.data[self.idx - 1])
        } else {
            Err(eyre!("unexpected EOF"))
        }
    }

    fn push(&mut self) {
        if !self.token.is_empty() {
            let token = Token {
                tok: Tok::from_str(&self.token.to_lowercase()).unwrap_or(Tok::Literal),
                s: self.token.clone(),
            };
            self.tokens.push(token);
            self.token.clear();
        }
    }
}
