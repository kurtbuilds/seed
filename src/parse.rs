use std::borrow::Cow;

#[derive(Debug)]
pub struct ParseError {
    message: Cow<'static, str>,
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "parse error: {}", self.message)
    }
}

#[derive(Debug)]
pub struct TokenStream<'a> {
    tokens: &'a [&'a str],
}

impl<'a> TokenStream<'a> {
    pub fn into_vec(self) -> Vec<&'a str> {
        self.tokens.to_vec()
    }

    pub fn clone(&self) -> Self {
        Self { tokens: self.tokens }
    }

    pub fn new(tokens: &'a [&'a str]) -> Self {
        Self { tokens }
    }

    pub fn next(&mut self) -> Option<&'a str> {
        if self.tokens.is_empty() {
            return None;
        }
        let t = self.tokens[0];
        self.tokens = &self.tokens[1..];
        Some(t)
    }

    pub fn peek(&self) -> Option<&'a str> {
        if self.tokens.is_empty() {
            return None;
        }
        Some(self.tokens[0])
    }

    pub fn next_if(&mut self, predicate: impl Fn(&str) -> bool) -> Option<&'a str> {
        if self.tokens.is_empty() {
            return None;
        }
        let t = self.tokens[0];
        if predicate(t) {
            self.tokens = &self.tokens[1..];
            Some(t)
        } else {
            None
        }
    }
}

impl ParseError {
    pub fn new(message: impl Into<Cow<'static, str>>) -> Self {
        Self { message: message.into() }
    }
}

pub trait Parse<'a>: Sized {
    fn parse(input: &mut TokenStream<'a>) -> Result<Self, ParseError>;
}

pub struct Punctuated<P, T> {
    delimiter: std::marker::PhantomData<P>,
    inner: Vec<T>,
}


impl<P, T> Punctuated<P, T> {
    pub fn new(inner: Vec<T>) -> Self {
        Self { inner, delimiter: std::marker::PhantomData }
    }

    pub fn into_vec(self) -> Vec<T> {
        self.inner
    }
}

impl<'a, P: Parse<'a>, T: Parse<'a>> Parse<'a> for Punctuated<P, T> {
    fn parse(input: &mut TokenStream<'a>) -> Result<Self, ParseError> {
        let mut tt = input.clone();
        let _ignore = P::parse(&mut tt);
        let mut inner = Vec::new();
        loop {
            let t = T::parse(&mut tt);
            match t {
                Ok(t) => inner.push(t),
                Err(_) => break,
            }
            let _ignore = P::parse(&mut tt);
        }
        *input = tt;
        Ok(Self::new(inner))
    }
}

pub struct Comma;

impl<'a> Parse<'a> for Comma {
    fn parse(input: &mut TokenStream<'a>) -> Result<Self, ParseError> {
        let mut tt = input.clone();
        let Some(t) = tt.next() else {
            return Err(ParseError::new("expected comma, got nothing"));
        };
        if t == "," {
            *input = tt;
            Ok(Self)
        } else {
            Err(ParseError::new("expected comma, got something else"))
        }
    }
}

pub struct Period;

impl<'a> Parse<'a> for Period {
    fn parse(input: &mut TokenStream<'a>) -> Result<Self, ParseError> {
        let mut tt = input.clone();
        let Some(t) = tt.next() else {
            return Err(ParseError::new("expected comma, got nothing"));
        };
        if t == "." {
            *input = tt;
            Ok(Self)
        } else {
            Err(ParseError::new("expected comma, got something else"))
        }
    }
}

pub struct Slash;

impl<'a> Parse<'a> for Slash {
    fn parse(input: &mut TokenStream<'a>) -> Result<Self, ParseError> {
        let mut tt = input.clone();
        let Some(t) = tt.next() else {
            return Err(ParseError::new("expected comma, got nothing"));
        };
        if t == "/" {
            *input = tt;
            Ok(Self)
        } else {
            Err(ParseError::new("expected comma, got something else"))
        }
    }
}

pub struct Gt;

impl<'a> Parse<'a> for Gt {
    fn parse(input: &mut TokenStream<'a>) -> Result<Self, ParseError> {
        let mut tt = input.clone();
        let Some(t) = tt.next() else {
            return Err(ParseError::new("expected >, got nothing"));
        };
        if t == ">" || t == "gt" {
            *input = tt;
            Ok(Self)
        } else {
            Err(ParseError::new("expected >, got something else"))
        }
    }
}

pub struct Lt;

impl<'a> Parse<'a> for Lt {
    fn parse(input: &mut TokenStream<'a>) -> Result<Self, ParseError> {
        let mut tt = input.clone();
        let Some(t) = tt.next() else {
            return Err(ParseError::new("expected <, got nothing"));
        };
        if t == "<" || t == "lt" {
            *input = tt;
            Ok(Self)
        } else {
            Err(ParseError::new("expected <, got something else"))
        }
    }
}

// constant value like 5
pub struct Constant<'a> {
    value: &'a str,
}

// literal value like "foo".
pub struct Literal<'a> {
    value: &'a str,
}

pub struct Sequence<T> {
    inner: Vec<T>,
}

pub struct Identifier<'a> {
    value: &'a str,
}

impl<'a> Parse<'a> for Identifier<'a> {
    fn parse(input: &mut TokenStream<'a>) -> Result<Self, ParseError> {
        let mut tt = input.clone();
        let Some(t) = tt.next() else {
            return Err(ParseError::new("expected identifier, got nothing"));
        };
        if t.chars().all(|c| c.is_alphanumeric() || c == '_') {
            *input = tt;
            Ok(Self { value: t })
        } else {
            Err(ParseError::new("expected identifier, got something else"))
        }
    }
}

impl<'a, T: Parse<'a>> Parse<'a> for Sequence<T> {
    fn parse(input: &mut TokenStream<'a>) -> Result<Self, ParseError> {
        let mut inner = Vec::new();
        loop {
            let t = T::parse(input);
            match t {
                Ok(t) => inner.push(t),
                Err(_) => break,
            }
        }
        Ok(Self { inner })
    }
}


pub fn lex(input: &[String]) -> Vec<&str> {
    let mut r = Vec::new();
    for input in input {
        let mut last = 0;
        for (i, ch) in input.char_indices() {
            if ch == '(' || ch == ')' || ch == ',' || ch == '/' {
                if last != i {
                    r.push(&input[last..i]);
                }
                r.push(&input[i..i + 1]);
                last = i + ch.len_utf8();
            }
        }
        if last < input.len() {
            r.push(&input[last..]);
        }
    }
    r
}

pub struct SelectionArgs {
    args: Vec<String>,
    tokens: Vec<&'static str>,
}

impl SelectionArgs {
    #[cfg(test)]
    pub fn from_shell(s: &str) -> Self {
        let args = shlex::split(s).unwrap();
        Self::new(args)
    }

    pub fn new(args: Vec<String>) -> SelectionArgs {
        let tokens = lex(&args);
        let tokens = unsafe { std::mem::transmute(tokens) };
        Self { args, tokens }
    }

    pub fn token_stream(&self) -> TokenStream<'_> {
        TokenStream::new(&self.tokens)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comma() {
        let mut tt = TokenStream::new(&[]);
        let c = Comma::parse(&mut tt);
        assert!(c.is_err());
    }

    #[test]
    fn test_delimited() {
        let mut tt = TokenStream::new(&[",", "foo", ",", "bar", ",", "baz"]);
        let d = Punctuated::<Comma, Identifier>::parse(&mut tt).unwrap();
        let d = d.into_vec();
        assert_eq!(d.len(), 3);
        assert_eq!(d[0].value, "foo");
        assert_eq!(d[1].value, "bar");
        assert_eq!(d[2].value, "baz");
    }

    #[test]
    fn test_lex() {
        let args = SelectionArgs::new(vec!["foo(bar,baz)".to_string()]);
        let tt = args.token_stream().into_vec();
        assert_eq!(tt, ["foo", "(", "bar", ",", "baz", ")"]);
    }
}




