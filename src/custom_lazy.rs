use std::marker::PhantomData;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The requested element is not an array")]
    NotAnArray,
    #[error("The requested element is not a number")]
    NotANumber,
    #[error("The requested element is not an object")]
    NotAnObject,
    #[error("The requested element is not a string")]
    NotAString,
    #[error("Unterminated array")]
    UnterminatedArray,
    #[error("Unterminated string")]
    UnterminatedString,
    #[error("The requested index {0} is not present in the array")]
    IndexNotFound(usize),
    #[error("Unsupported character in number: {0}")]
    UnsupportedCharInNumber(char),
    #[error("Unexpected value: {0}")]
    UnexpectedValue(char),
    #[error("Unterminated object")]
    UnterminatedObject,
    #[error("Key was not found: {0}")]
    KeyNotFound(std::string::String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
struct Cursor<'a> {
    iter: std::str::Chars<'a>,
    current_char: Option<char>,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a str) -> Self {
        let mut iter = data.chars();
        // Initialize the cursor to point to the first non-whitespace character
        let mut current_char = iter.next();
        while current_char.is_some_and(|c| c.is_ascii_whitespace()) {
            current_char = iter.next();
        }
        Self { iter, current_char }
    }

    fn advance_character(&mut self) {
        self.current_char = self.iter.next();
    }

    fn advance_token(&mut self) {
        self.advance_character();
        while self.current_char.is_some_and(|c| c.is_ascii_whitespace()) {
            self.advance_character()
        }
    }

    fn advance_value(&mut self) -> Result<()> {
        let Some(current_char) = self.current_char else {
            // End of file
            return Ok(());
        };

        match current_char {
            '{' => {
                self.advance_object()?;
            }
            '"' => {
                self.advance_string()?;
            }
            '-' => {
                self.advance_number()?;
            }
            c if c.is_ascii_digit() => {
                self.advance_number()?;
            }
            // These are unimplemented because they are not needed for the API response
            '[' => {
                todo!();
            }
            't' => {
                todo!();
            }
            'f' => {
                todo!();
            }
            'n' => {
                todo!();
            }
            _ => return Err(Error::UnexpectedValue(current_char)),
        }

        Ok(())
    }

    fn advance_object(&mut self) -> Result<()> {
        self.advance_token();

        loop {
            let Some(current_char) = self.current_char else {
                return Err(Error::UnterminatedObject);
            };

            if current_char == '}' {
                self.advance_token();
                return Ok(());
            }

            self.advance_string()?;
            if self.current_char.is_some_and(|c| c != ':') {
                return Err(Error::UnterminatedObject);
            }
            self.advance_token();

            self.advance_value()?;

            if self.current_char.is_some_and(|c| c == ',') {
                self.advance_token();
            }
        }
    }

    fn advance_string(&mut self) -> Result<()> {
        self.advance_character();

        loop {
            let Some(current_char) = self.current_char else {
                return Err(Error::UnterminatedString);
            };

            if current_char == '"' {
                self.advance_token();
                return Ok(());
            }

            if current_char == '\\' {
                self.advance_character();
                if self.current_char.is_some_and(|c| c == 'u') {
                    // skip `\uABCD` patterns
                    for _ in 0..4 {
                        self.advance_character();
                    }
                }
            }

            self.advance_character();
        }
    }

    fn advance_and_match_string(&mut self, s: &str) -> Result<bool> {
        self.advance_character();

        let mut s_iter = s.chars();
        let mut matches = true;

        loop {
            let Some(current_char) = self.current_char else {
                return Err(Error::UnterminatedString);
            };

            if current_char == '"' {
                matches &= s_iter.next().is_none();
                self.advance_token();
                return Ok(matches);
            }

            if current_char == '\\' {
                todo!("string escape support is unimplemented");
            }

            matches &= s_iter.next().is_some_and(|c| current_char == c);

            self.advance_character();
        }
    }

    fn advance_and_get_string(&mut self) -> Result<&'a str> {
        let start = self.iter.clone();

        self.advance_character();

        let mut numbytes = 0;

        loop {
            let Some(current_char) = self.current_char else {
                return Err(Error::UnterminatedString);
            };

            if current_char == '"' {
                self.advance_token();
                let substr = &start.as_str()[..numbytes];
                return Ok(substr);
            }

            if current_char == '\\' {
                todo!("string escape support is unimplemented");
            }

            numbytes += current_char.len_utf8();

            self.advance_character();
        }
    }

    fn advance_number(&mut self) -> Result<()> {
        // we already know the first char is belongs to the number, ignore it
        self.advance_character();

        loop {
            let Some(current_char) = self.current_char else {
                return Ok(());
            };

            match current_char {
                c if c.is_ascii_digit() => {}
                // These characters can be part of the number. For now we will assume we have only
                // integral numbers, for simplicity.
                c @ ('.' | 'e' | 'E' | '+' | '-') => return Err(Error::UnsupportedCharInNumber(c)),
                _ => {
                    while self.current_char.is_some_and(|c| c.is_ascii_whitespace()) {
                        self.advance_character();
                    }
                    return Ok(());
                }
            }

            self.advance_character();
        }
    }

    fn get_char(&self) -> Option<char> {
        self.current_char
    }
}

pub struct Node<'a, T> {
    cursor: Cursor<'a>,
    _pd: std::marker::PhantomData<T>,
}

pub struct DocumentType {}
pub struct ArrayType {}
pub struct ObjectType {}
pub struct StringType {}
pub struct NumberType {}
pub struct UnknownType {}

pub type Document<'a> = Node<'a, DocumentType>;
pub type GenericNode<'a> = Node<'a, UnknownType>;
pub type Array<'a> = Node<'a, ArrayType>;
pub type Object<'a> = Node<'a, ObjectType>;
pub type String<'a> = Node<'a, StringType>;
pub type Number<'a> = Node<'a, NumberType>;
// Note that Bool and Null types are missing because they are not needed for the purposes of the
// challenge.

impl<'a> Document<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            cursor: Cursor::new(data),
            _pd: std::marker::PhantomData,
        }
    }

    /// Note that there would be more methods to cast the current document to other object types.
    /// For simplicity, given that they are not needed for this challenge, I did not add them.
    pub fn as_array(self) -> Result<Array<'a>> {
        if self.cursor.get_char().is_none_or(|c| c != '[') {
            return Err(Error::NotAnArray);
        }
        Ok(Array {
            cursor: self.cursor.clone(),
            _pd: PhantomData,
        })
    }
}

// Array should implement an iterator interface that allows us to visit every node.
// Especially because we cannot tell the length of the array beforehand, it would be
// useful.
impl<'a> Array<'a> {
    pub fn get_index(&self, index: usize) -> Result<GenericNode<'a>> {
        let mut cursor = self.cursor.clone();
        cursor.advance_token();

        for _ in 0..index {
            let Some(current_char) = cursor.current_char else {
                return Err(Error::UnterminatedArray);
            };

            if current_char == ']' {
                return Err(Error::IndexNotFound(index));
            }

            cursor.advance_value()?;

            if cursor.current_char.is_some_and(|c| c == ',') {
                cursor.advance_token();
            }
        }

        Ok(GenericNode {
            cursor,
            _pd: PhantomData,
        })
    }
}

impl<'a> GenericNode<'a> {
    pub fn as_array(self) -> Result<Array<'a>> {
        if self.cursor.get_char().is_none_or(|c| c != '[') {
            return Err(Error::NotAnArray);
        }
        Ok(Array {
            cursor: self.cursor.clone(),
            _pd: PhantomData,
        })
    }

    pub fn as_object(self) -> Result<Object<'a>> {
        if self.cursor.get_char().is_none_or(|c| c != '{') {
            return Err(Error::NotAnObject);
        }
        Ok(Object {
            cursor: self.cursor.clone(),
            _pd: PhantomData,
        })
    }

    pub fn as_number(self) -> Result<Number<'a>> {
        if self
            .cursor
            .get_char()
            .is_none_or(|c| c != '-' && !c.is_ascii_digit())
        {
            return Err(Error::NotANumber);
        }
        Ok(Number {
            cursor: self.cursor.clone(),
            _pd: PhantomData,
        })
    }

    pub fn as_string(self) -> Result<String<'a>> {
        if self.cursor.get_char().is_none_or(|c| c != '"') {
            return Err(Error::NotAString);
        }
        Ok(String {
            cursor: self.cursor.clone(),
            _pd: PhantomData,
        })
    }
}

impl<'a> Object<'a> {
    pub fn get_key(&self, s: &str) -> Result<GenericNode<'a>> {
        let mut cursor = self.cursor.clone();
        cursor.advance_token();

        loop {
            if cursor.current_char.is_some_and(|c| c == '}') {
                return Err(Error::KeyNotFound(s.to_string()));
            }

            let match_found = cursor.advance_and_match_string(s)?;

            if cursor.current_char.is_none_or(|c| c != ':') {
                return Err(Error::UnterminatedObject);
            }
            cursor.advance_token();

            if match_found {
                return Ok(GenericNode {
                    cursor,
                    _pd: PhantomData,
                });
            } else {
                cursor.advance_value()?;
            }

            if cursor.current_char.is_some_and(|c| c == ',') {
                cursor.advance_token();
            }
        }
    }
}

impl Number<'_> {
    pub fn get_value(&self) -> u64 {
        todo!()
    }
}

impl<'a> String<'a> {
    pub fn get_value(&self) -> Result<&'a str> {
        let mut cursor = self.cursor.clone();
        cursor.advance_and_get_string()
    }

    pub fn get_value_as_f64(&self) -> Result<f64> {
        self.get_value()?.parse().map_err(|_| Error::NotANumber)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn cursor_number() {
        let mut cursor = Cursor::new("1234543, ");
        cursor.advance_value().unwrap();

        assert!(cursor.current_char.is_some_and(|c| c == ','));

        let mut cursor = Cursor::new("-1234543, ");
        cursor.advance_value().unwrap();

        assert!(cursor.current_char.is_some_and(|c| c == ','));
    }

    #[test]
    fn cursor_string() {
        let mut cursor = Cursor::new(r##""This is a string with escaped \" characters","##);
        cursor.advance_value().unwrap();

        assert!(cursor.current_char.is_some_and(|c| c == ','));
    }

    #[test]
    fn cursor_object() {
        let mut cursor = Cursor::new(r##"{"key":-124,"key2":"","key3":1544}."##);
        cursor.advance_value().unwrap();

        assert!(cursor.current_char.is_some_and(|c| c == '.'));
    }

    #[test]
    fn document_api() {
        let doc = Document::new(r###"[ {"a": 52, "b" : "c"}, 3 ]"###);

        let array = doc.as_array().unwrap();

        let index0 = array.get_index(0).unwrap();
        assert!(index0.cursor.current_char.is_some_and(|c| c == '{'));

        let index1 = array.get_index(1).unwrap();
        assert!(index1.cursor.current_char.is_some_and(|c| c == '3'));

        let object = index0.as_object().unwrap();
        let _number = index1.as_number().unwrap();

        let inner_value = object.get_key("a").unwrap();

        assert!(inner_value.cursor.current_char.is_some_and(|c| c == '5'));
        let _number = inner_value.as_number();

        let inner_value = object.get_key("b").unwrap();

        let c = inner_value.as_string().unwrap().get_value().unwrap();
        assert_eq!(c, "c");
    }
}
