//! Bespoke json deserializer, because serde is great, but doesn't give you access to the raw input
//! bytes of an object

use std::mem::MaybeUninit;

use crate::utils::{LazyF64, LazyU64};

#[derive(thiserror::Error, Debug, Clone)]
/// The error type used for the json module.
pub enum Error {
    #[error("Unterminated string")]
    UnterminatedString,
    #[error("Invalid identifier")]
    InvalidIdentifier,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Unexpected token")]
    UnexpectedToken,
    #[error("Empty JSON")]
    EmptyJson,
    #[error("Unterminated JSON Object")]
    UnterminatedJsonObject,
    #[error("Malformed JSON Object")]
    MalformedJsonObject,
    #[error("Unterminated JSON Array")]
    UnterminatedJsonArray,
    #[error("Malformed JSON Array")]
    MalformedJsonArray,
    #[error("The requested key was not found in the object")]
    KeyNotFound,
    #[error("The top-level is not a JSON array")]
    NotAJsonArray,
    #[error("Expected a JSON ticket price entry, but found something else")]
    NotAJsonObject,
    #[error("Unknown field {0}")]
    UnknownField(String),
    #[error("Cannot get value out of non-string and non-integer type")]
    InvalidTokenType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenType {
    Lbrace,
    Rbrace,
    Colon,
    Comma,
    Number,
    String,
    Lbracket,
    Rbracket,
    Null,
    True,
    False,
}

#[derive(Debug, Clone)]
struct Span(usize, usize);

impl Span {
    const fn at(pos: usize) -> Self {
        Self(pos, pos + 1)
    }

    const fn range(start: usize, end: usize) -> Self {
        Self(start, end)
    }

    pub fn extend(&self, other: &Self) -> Self {
        Self(
            std::cmp::min(self.0, other.0),
            std::cmp::max(self.1, other.1),
        )
    }
}

#[derive(Debug, Clone)]
struct Token {
    span: Span,
    ty: TokenType,
}

fn char_to_token_type(c: char) -> TokenType {
    match c {
        '{' => TokenType::Lbrace,
        '}' => TokenType::Rbrace,
        '[' => TokenType::Lbracket,
        ']' => TokenType::Rbracket,
        ':' => TokenType::Colon,
        ',' => TokenType::Comma,
        _ => panic!("Single character does not match a token: {c}"),
    }
}

fn ident_to_token_type(ident: &str) -> TokenType {
    match ident {
        "null" => TokenType::Null,
        "true" => TokenType::True,
        "false" => TokenType::False,
        _ => panic!("Invalid identifier: {ident}"),
    }
}

struct TokenIter<'a> {
    iter: std::iter::Peekable<std::iter::Enumerate<std::str::Chars<'a>>>,
}

impl<'a> TokenIter<'a> {
    fn new(s: &'a str) -> Self {
        TokenIter {
            iter: s.chars().enumerate().peekable(),
        }
    }

    fn next_string(&mut self) -> Result<Token, Error> {
        let (start, _) = self
            .iter
            .next()
            .expect("At least first character must be present");

        while let Some((i, c)) = self.iter.next() {
            match c {
                '\\' => {
                    // escape sequence
                    if self.iter.next().is_some_and(|(_, c)| c == 'u') {
                        for _ in 0..4 {
                            // consume 4 hex digits
                            self.iter.next();
                        }
                    }
                }
                '"' => {
                    return Ok(Token {
                        span: Span::range(start, i + 1),
                        ty: TokenType::String,
                    })
                }
                _ => {}
            }
        }

        Err(Error::UnterminatedString)
    }

    fn next_number(&mut self) -> Result<Token, Error> {
        let (start, _) = self
            .iter
            .next()
            .expect("At least first character must be present");

        let mut end = start;

        while let Some((i, c)) = self.iter.peek() {
            match c {
                '0'..='9' | '.' | 'e' | 'E' => {
                    end = *i;
                    self.iter.next();
                }
                _ => {
                    break;
                }
            }
        }

        Ok(Token {
            span: Span::range(start, end + 1),
            ty: TokenType::Number,
        })
    }

    fn next_ident(&mut self, ident: &str) -> Result<Token, Error> {
        let start = self
            .iter
            .peek()
            .map(|(i, _)| *i)
            .expect("At least first character must be present");

        for expected in ident.chars() {
            let matches = self
                .iter
                .next()
                .is_some_and(|(_, actual)| actual == expected);
            if !matches {
                return Err(Error::InvalidIdentifier);
            }
        }

        Ok(Token {
            span: Span::range(start, start + ident.len()),
            ty: ident_to_token_type(ident),
        })
    }
}

impl Iterator for TokenIter<'_> {
    type Item = Result<Token, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (i, c) = self.iter.peek().map(|(i, c)| (*i, *c))?;

            match c {
                ' ' | '\n' | '\r' | '\t' => {
                    self.iter.next();
                }
                '{' | '}' | '[' | ']' | ':' | ',' => {
                    self.iter.next();
                    return Some(Ok(Token {
                        span: Span::at(i),
                        ty: char_to_token_type(c),
                    }));
                }
                '"' => {
                    return Some(self.next_string());
                }
                'n' => {
                    return Some(self.next_ident("null"));
                }
                't' => {
                    return Some(self.next_ident("true"));
                }
                'f' => {
                    return Some(self.next_ident("false"));
                }
                c if c.is_ascii_digit() || c == '-' => {
                    return Some(self.next_number());
                }
                _ => {
                    return Some(Err(Error::InvalidToken));
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
/// A JSON object as an AST element.
pub struct ObjectAst {
    span: Span,
    elems: Vec<(StringAst, ValueAst)>,
}

impl ObjectAst {
    /// Obtains the requested item by key, or returns KeyNotFound if the key was not found in the
    /// object.
    pub fn get_item(&self, s: &str, key: &str) -> Result<&ValueAst, Error> {
        for (k, v) in &self.elems {
            if k.value(s) == key {
                return Ok(v);
            }
        }
        Err(Error::KeyNotFound)
    }

    /// Returns the raw json data that makes up this object.
    pub fn get_raw_string<'a>(&self, s: &'a str) -> &'a str {
        let begin = self.span.0;
        let end = self.span.1;
        &s[begin..end]
    }
}

#[derive(Debug, Clone)]
/// A Json array as an AST element.
pub struct ArrayAst {
    span: Span,
    elems: Vec<ValueAst>,
}

impl ArrayAst {
    /// Returns the raw json data that makes up this object.
    pub fn get_raw_string<'a>(&self, s: &'a str) -> &'a str {
        let begin = self.span.0;
        let end = self.span.1;
        &s[begin..end]
    }
}

impl IntoIterator for ArrayAst {
    type Item = ValueAst;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.elems.into_iter()
    }
}

#[derive(Debug, Clone)]
/// A JSON number as an AST element
pub struct NumberAst(Token);

impl NumberAst {
    /// Returns the raw json data that makes up this object.
    pub fn get_raw_string<'a>(&self, s: &'a str) -> &'a str {
        let begin = self.0.span.0;
        let end = self.0.span.1;
        &s[begin..end]
    }
}

#[derive(Debug, Clone)]
/// A JSON string as an AST element
pub struct StringAst(Token);

impl StringAst {
    /// Returns the inner string inside the json object.
    pub fn value<'a>(&self, s: &'a str) -> &'a str {
        let begin = self.0.span.0;
        let end = self.0.span.1;
        &s[begin + 1..end - 1]
    }

    /// Returns the raw json data that makes up this object.
    pub fn get_raw_string<'a>(&self, s: &'a str) -> &'a str {
        let begin = self.0.span.0;
        let end = self.0.span.1;
        &s[begin..end]
    }
}

#[derive(Debug, Clone)]
pub struct NullAst(Token);

impl NullAst {
    /// Returns the raw json data that makes up this object.
    pub fn get_raw_string<'a>(&self, s: &'a str) -> &'a str {
        let begin = self.0.span.0;
        let end = self.0.span.1;
        &s[begin..end]
    }
}

#[derive(Debug, Clone)]
pub struct BoolAst(Token);

impl BoolAst {
    /// Returns the raw json data that makes up this object.
    pub fn get_raw_string<'a>(&self, s: &'a str) -> &'a str {
        let begin = self.0.span.0;
        let end = self.0.span.1;
        &s[begin..end]
    }
}

#[derive(Debug, Clone)]
/// A Json Value as an AST element.
pub enum ValueAst {
    Object(ObjectAst),
    Array(ArrayAst),
    Number(NumberAst),
    String(StringAst),
    Null(NullAst),
    Bool(BoolAst),
}

impl ValueAst {
    /// Returns the raw json data that makes up this object.
    pub fn get_raw_string<'a>(&self, s: &'a str) -> &'a str {
        match self {
            ValueAst::Object(o) => o.get_raw_string(s),
            ValueAst::Array(a) => a.get_raw_string(s),
            ValueAst::Null(n) => n.get_raw_string(s),
            ValueAst::Number(n) => n.get_raw_string(s),
            ValueAst::String(str) => str.get_raw_string(s),
            ValueAst::Bool(b) => b.get_raw_string(s),
        }
    }

    /// Returns the inner value in hte json for
    pub fn get_str_or_number_value<'a>(&self, s: &'a str) -> Result<&'a str, Error> {
        Ok(match self {
            ValueAst::Number(n) => n.get_raw_string(s),
            ValueAst::String(str) => str.value(s),
            _ => {
                return Err(Error::InvalidTokenType);
            }
        })
    }
}

/// Parses the given JSON data and returns an AST that represents the JSON data.
pub fn parse_json(s: &str) -> Result<ValueAst, Error> {
    fn parse_json_inner(iter: &mut std::iter::Peekable<TokenIter>) -> Result<ValueAst, Error> {
        let Some(token) = iter.next() else {
            return Err(Error::EmptyJson);
        };

        let token = token?;

        match token.ty {
            TokenType::Number => Ok(ValueAst::Number(NumberAst(token))),
            TokenType::String => Ok(ValueAst::String(StringAst(token))),
            TokenType::Null => Ok(ValueAst::Null(NullAst(token))),
            TokenType::True => Ok(ValueAst::Bool(BoolAst(token))),
            TokenType::False => Ok(ValueAst::Bool(BoolAst(token))),
            TokenType::Lbrace => {
                // Handle object
                let mut object = vec![];

                let lbrace = token.span;

                let next = iter.peek().ok_or(Error::UnterminatedJsonObject)?;
                let next = next.clone()?;
                if next.ty == TokenType::Rbrace {
                    iter.next();
                    let span = lbrace.extend(&next.span);
                    return Ok(ValueAst::Object(ObjectAst {
                        elems: object,
                        span,
                    }));
                }

                while let Some(next) = iter.next() {
                    let key = next?;
                    if key.ty != TokenType::String {
                        return Err(Error::MalformedJsonObject);
                    };

                    if iter.next().ok_or(Error::UnterminatedJsonObject)??.ty != TokenType::Colon {
                        return Err(Error::MalformedJsonObject);
                    }

                    let value = parse_json_inner(iter)?;
                    object.push((StringAst(key), value));

                    let next = iter.next().ok_or(Error::UnterminatedJsonObject)?;
                    let next = next?;
                    match next.ty {
                        TokenType::Comma => {}
                        TokenType::Rbrace => {
                            let span = lbrace.extend(&next.span);
                            return Ok(ValueAst::Object(ObjectAst {
                                elems: object,
                                span,
                            }));
                        }
                        _ => {
                            return Err(Error::MalformedJsonObject);
                        }
                    }
                }
                Err(Error::UnterminatedJsonObject)
            }
            TokenType::Lbracket => {
                // Handle array
                let mut array = vec![];
                let lbracket = token.span;

                let next = iter.peek().ok_or(Error::UnterminatedJsonArray)?;
                let next = next.clone()?;
                if next.ty == TokenType::Rbracket {
                    iter.next();
                    let span = lbracket.extend(&next.span);
                    return Ok(ValueAst::Array(ArrayAst { elems: array, span }));
                }

                loop {
                    let value = parse_json_inner(iter)?;
                    array.push(value);

                    let next = iter.next().ok_or(Error::UnterminatedJsonArray)?;
                    let next = next?;

                    match next.ty {
                        TokenType::Rbracket => {
                            let span = lbracket.extend(&next.span);
                            return Ok(ValueAst::Array(ArrayAst { span, elems: array }));
                        }
                        TokenType::Comma => {}
                        _ => {
                            return Err(Error::MalformedJsonArray);
                        }
                    }
                }
            }
            _ => Err(Error::UnexpectedToken),
        }
    }
    let mut iter = TokenIter::new(s).peekable();
    parse_json_inner(&mut iter)
}

#[derive(Debug)]
pub struct PriceChange24Hr<'a> {
    pub symbol: &'a str,
    pub price_change: LazyF64<'a>,
    pub price_change_percent: LazyF64<'a>,
    pub last_price: LazyF64<'a>,
    pub last_qty: LazyF64<'a>,
    pub open: LazyF64<'a>,
    pub high: LazyF64<'a>,
    pub low: LazyF64<'a>,
    pub volume: LazyF64<'a>,
    pub amount: LazyF64<'a>,
    pub bid_price: LazyF64<'a>,
    pub ask_price: LazyF64<'a>,
    pub open_time: LazyU64<'a>,
    pub close_time: LazyU64<'a>,
    pub first_trade_id: LazyU64<'a>,
    pub trade_count: LazyU64<'a>,
    pub strike_price: LazyF64<'a>,
    pub exercise_price: LazyF64<'a>,
}

pub fn parse_price_change_entry<'a>(
    s: &'a str,
    value: &ValueAst,
) -> Result<PriceChange24Hr<'a>, Error> {
    let mut entry: MaybeUninit<PriceChange24Hr> = MaybeUninit::zeroed();
    let entry_ptr = entry.as_mut_ptr();

    match value {
        ValueAst::Object(object) => {
            for (k, v) in &object.elems {
                let v = v.get_str_or_number_value(s)?;
                match k.value(s) {
                    "symbol" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).symbol).write(v);
                    },
                    "priceChange" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).price_change).write(LazyF64(v));
                    },
                    "priceChangePercent" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).price_change_percent).write(LazyF64(v));
                    },
                    "lastPrice" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).last_price).write(LazyF64(v));
                    },
                    "lastQty" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).last_qty).write(LazyF64(v));
                    },
                    "open" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).open).write(LazyF64(v));
                    },
                    "high" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).high).write(LazyF64(v));
                    },
                    "low" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).low).write(LazyF64(v));
                    },
                    "volume" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).volume).write(LazyF64(v));
                    },
                    "amount" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).amount).write(LazyF64(v));
                    },
                    "bidPrice" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).bid_price).write(LazyF64(v));
                    },
                    "askPrice" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).ask_price).write(LazyF64(v));
                    },
                    "openTime" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).open_time).write(LazyU64(v));
                    },
                    "closeTime" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).close_time).write(LazyU64(v));
                    },
                    "firstTradeId" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).first_trade_id).write(LazyU64(v));
                    },
                    "tradeCount" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).trade_count).write(LazyU64(v));
                    },
                    "strikePrice" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).strike_price).write(LazyF64(v));
                    },
                    "exercisePrice" => unsafe {
                        std::ptr::addr_of_mut!((*entry_ptr).exercise_price).write(LazyF64(v));
                    },
                    k => {
                        return Err(Error::UnknownField(k.to_string()));
                    }
                }
            }

            unsafe { Ok(entry.assume_init()) }
        }
        _ => Err(Error::NotAJsonArray),
    }
}

pub fn parse(s: &str) -> Result<Vec<PriceChange24Hr>, Error> {
    let data = parse_json(s)?;

    match data {
        ValueAst::Array(array) => {
            let mut result = Vec::with_capacity(array.elems.len());
            for elem in array.elems {
                result.push(parse_price_change_entry(s, &elem)?);
            }
            Ok(result)
        }
        _ => Err(Error::NotAJsonArray),
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_tokenization() {
        let mut iter = TokenIter::new(r#"{ "type": 1, } [ ]"#);

        assert!(matches!(
            iter.next(),
            Some(Ok(Token {
                span: Span(0, 1),
                ty: TokenType::Lbrace
            }))
        ));

        assert!(matches!(
            iter.next(),
            Some(Ok(Token {
                span: Span(2, 8),
                ty: TokenType::String
            }))
        ));

        assert!(matches!(
            iter.next(),
            Some(Ok(Token {
                span: Span(8, 9),
                ty: TokenType::Colon
            }))
        ));

        assert!(matches!(
            iter.next(),
            Some(Ok(Token {
                span: Span(10, 11),
                ty: TokenType::Number
            }))
        ));

        assert!(matches!(
            iter.next(),
            Some(Ok(Token {
                span: Span(11, 12),
                ty: TokenType::Comma
            }))
        ));

        assert!(matches!(
            iter.next(),
            Some(Ok(Token {
                span: Span(13, 14),
                ty: TokenType::Rbrace
            }))
        ));

        assert!(matches!(
            iter.next(),
            Some(Ok(Token {
                span: Span(15, 16),
                ty: TokenType::Lbracket
            }))
        ));

        assert!(matches!(
            iter.next(),
            Some(Ok(Token {
                span: Span(17, 18),
                ty: TokenType::Rbracket
            }))
        ));

        assert!(matches!(iter.next(), None));
    }

    #[test]
    fn test_parsing_string() {
        let input = r#""Hello!", {}"#;
        let value = parse_json(input).unwrap();

        assert!(matches!(
            value,
            ValueAst::String(StringAst(Token {
                span: Span(0, 8),
                ty: TokenType::String
            }))
        ));

        match value {
            ValueAst::String(s) => {
                assert_eq!(s.value(input), "Hello!");
            }
            _ => panic!("Unexpected type!"),
        }
    }

    #[test]
    fn test_parsing_number() {
        let value = parse_json(r#"-1234.324,"#).unwrap();

        assert!(matches!(
            value,
            ValueAst::Number(NumberAst(Token {
                span: Span(0, 9),
                ty: TokenType::Number
            }))
        ));
    }

    #[test]
    fn test_parsing_true() {
        let value = parse_json(r#"true,"#).unwrap();

        assert!(matches!(
            value,
            ValueAst::Bool(BoolAst(Token {
                span: Span(0, 4),
                ty: TokenType::True
            }))
        ));
    }

    #[test]
    fn test_parsing_false() {
        let value = parse_json(r#"false,"#).unwrap();

        assert!(matches!(
            value,
            ValueAst::Bool(BoolAst(Token {
                span: Span(0, 5),
                ty: TokenType::False
            }))
        ));
    }

    #[test]
    fn test_parsing_null() {
        let value = parse_json(r#"null,"#).unwrap();

        assert!(matches!(
            value,
            ValueAst::Null(NullAst(Token {
                span: Span(0, 4),
                ty: TokenType::Null
            }))
        ));
    }

    #[test]
    fn test_parsing_array() {
        let value = parse_json(r#"[null, true, false, "test", 123], {}"#).unwrap();

        let ValueAst::Array(array) = value else {
            panic!("Value is not an array: {value:?}");
        };

        assert_eq!(array.span.0, 0);
        assert_eq!(array.span.1, 32);

        assert_eq!(array.elems.len(), 5);

        assert!(matches!(
            array.elems[0],
            ValueAst::Null(NullAst(Token {
                span: Span(1, 5),
                ty: TokenType::Null
            }))
        ));

        assert!(matches!(
            array.elems[1],
            ValueAst::Bool(BoolAst(Token {
                span: Span(7, 11),
                ty: TokenType::True
            }))
        ));

        assert!(matches!(
            array.elems[2],
            ValueAst::Bool(BoolAst(Token {
                span: Span(13, 18),
                ty: TokenType::False
            }))
        ));

        assert!(matches!(
            array.elems[3],
            ValueAst::String(StringAst(Token {
                span: Span(20, 26),
                ty: TokenType::String
            }))
        ));

        assert!(matches!(
            array.elems[4],
            ValueAst::Number(NumberAst(Token {
                span: Span(28, 31),
                ty: TokenType::Number
            }))
        ));
    }

    #[test]
    fn test_parsing_object() {
        let value = parse_json(r#"{"hi":null, "hello":true}"#).unwrap();

        let ValueAst::Object(object) = value else {
            panic!("Value is not an object: {value:?}");
        };

        assert_eq!(object.span.0, 0);
        assert_eq!(object.span.1, 25);

        assert_eq!(object.elems.len(), 2);

        assert!(matches!(
            object.elems[0],
            (
                StringAst(Token {
                    span: Span(1, 5),
                    ty: TokenType::String
                }),
                ValueAst::Null(NullAst(Token {
                    span: Span(6, 10),
                    ty: TokenType::Null
                }))
            )
        ));

        assert!(matches!(
            object.elems[1],
            (
                StringAst(Token {
                    span: Span(12, 19),
                    ty: TokenType::String
                }),
                ValueAst::Bool(BoolAst(Token {
                    span: Span(20, 24),
                    ty: TokenType::True
                }))
            )
        ));
    }
}
