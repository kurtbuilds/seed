use crate::parse::{Comma, Parse, ParseError, Punctuated, TokenStream};

// #[derive(Debug, PartialEq)]
// pub enum Selector {
//     Id(String),
//     Rand(usize),
//     Limit(usize),
//     Latest,
//     Sort(String, bool),
//     Expr(sqlmo::Expr),
//     /// refers to the index of a previous selection in the list.
//     /// e.g. vec![Selection { "org", "123" }, Selection { "deduction", ForeignKey(0) }]
//     /// would mean "select all deductions from org 123"
//     ForeignKey(usize),
//     Prompt,
// }

#[derive(Debug, PartialEq)]
pub struct Selection {
    pub table: String,
    pub selectors: Vec<Selector>,
}

pub struct ParseSelection<'a> {
    table: &'a str,
    selectors: Vec<ParseSelector<'a>>,
}


impl From<ParseSelection<'_>> for Selection {
    fn from(p: ParseSelection) -> Self {
        let table = p.table.to_string();
        let selectors = p.selectors.into_iter().map(Into::into).collect();
        Self { table, selectors }
    }
}


impl<'a> Parse<'a> for ParseSelection<'a> {
    fn parse(input: &mut TokenStream<'a>) -> Result<ParseSelection<'a>, ParseError> {
        let mut tt = input.clone();
        let table = tt.next().ok_or(ParseError::new("expected table name"))?;
        let selectors = Punctuated::<Comma, ParseSelector>::parse(&mut tt)?;
        *input = tt;
        Ok(Self { table, selectors: selectors.into_vec() })
    }
}

#[derive(Debug, PartialEq)]
pub enum Selector {
    Id(String),
    Rand(usize),
    Limit(usize),
    Latest(usize),
    // asc = false, desc = true
    Sort(String, bool),
    Expr,
}

pub struct ParseSelector<'a> {
    selector: &'a str,
    args: Vec<&'a str>,
}

impl<'a> Parse<'a> for ParseSelector<'a> {
    fn parse(input: &mut TokenStream<'a>) -> Result<Self, ParseError> {
        let mut tt = input.clone();
        let selector = tt.next().ok_or(ParseError::new("expected selector"))?;
        if selector == "/" {
            return Err(ParseError::new("expected selector, got /"));
        }
        let mut args = Vec::new();
        while let Some(t) = tt.next_if(|t| t != "," && t != "/") {
            args.push(t);
        }
        *input = tt;
        Ok(Self { selector, args })
    }
}

impl From<ParseSelector<'_>> for Selector {
    fn from(p: ParseSelector) -> Self {
        if p.args.is_empty() {
            Selector::Id(p.selector.to_string())
        } else if p.selector == "rand" {
            Selector::Rand(p.args[0].parse().unwrap())
        } else if p.selector == "latest" {
            Selector::Latest(p.args[0].parse().unwrap())
        } else if p.selector == "limit" {
            Selector::Limit(p.args[0].parse().unwrap())
        } else if p.selector == "sort" {
            let direction = p.args.get(1).map(|&s| s == "desc").unwrap_or(false);
            Selector::Sort(p.args[0].to_string(), direction)
        } else {
            Selector::Expr
        }
    }
}

pub struct MultiSelection(pub Vec<Selection>);

#[cfg(test)]
mod tests {
    use crate::parse::{SelectionArgs, Slash};
    use super::*;

    #[test]
    fn test_parse_selection() {
        let mut tt = TokenStream::new(&["org", "123"]);
        let s = ParseSelection::parse(&mut tt).unwrap();
        let s: Selection = s.into();
        assert_eq!(s.table, "org");
        assert_eq!(s.selectors.len(), 1);
        let Selector::Id(s) = &s.selectors[0] else {
            panic!();
        };
        assert_eq!(s, "123");
    }

    #[test]
    fn test_parse_selector2() {
        let args = SelectionArgs::from_shell("org 123 / deduction latest 1000");
        let s: Vec<Selection> = Punctuated::<Slash, ParseSelection>::parse(&mut args.token_stream()).unwrap().into_vec()
            .into_iter()
            .map(Into::into)
            .collect();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].table, "org");
        assert_eq!(s[1].table, "deduction");
        assert_eq!(s[1].selectors.len(), 1);
        let Selector::Latest(n) = &s[1].selectors[0] else {
            panic!();
        };
    }
}
