//! Lexer for the config.
//!
//! Use [lex] to lex some input.

use token::*;
use winnow::{
    ascii::multispace0,
    combinator::{delimited, repeat},
    error::ParserError,
    prelude::*,
    token::take_until,
};

/// A value with a type
#[derive(Debug, PartialEq, Eq)]
pub enum Value<'src> {
    /// A string
    String(&'src str),
    /// A regex
    Regex(&'src str),
    /// A number
    Number(&'src str),
    /// A boolean
    Boolean(&'src str),
    /// A color
    Color(&'src str),
}

/// A token of the config format
#[derive(Debug, PartialEq, Eq)]
pub enum Token<'src> {
    /// Opening-symbol for a block
    BlockOpen,
    /// Closing-symbol for a block
    BlockClose,
    /// An identifier
    Identifier(&'src str),
    /// A property-separator
    Separator,
    /// A value; see [Value]
    Value(Value<'src>),
    /// A comment, either single- or multiline
    Comment(&'src str),
}

/// Lexes a [str] into a [Vec] of [Token]s,
/// or returns an [Err] if lexing failed.
pub fn lex(
    input: &str,
) -> Result<Vec<Token>, winnow::error::ParseError<&str, winnow::error::ContextError>> {
    repeat(0.., delimited(multispace0, token, multispace0)).parse(input)
}

pub fn delimited_by<'src, E: ParserError<&'src str>>(
    start: &'static str,
    end: &'static str,
) -> impl Parser<&'src str, &'src str, E> {
    delimited(start, take_until(0.., end), end)
}

/// Lexers to lex individual [Token]s.
///
/// All lexers assume their token starts immediately (i.e., no preceding whitespace)
/// and will not consume any subsequent whitespace,
/// except when that whitespace is used as a delimiter,
/// in which case it will be noted.
pub mod token {
    use super::*;
    use winnow::{
        ascii::{line_ending, till_line_ending},
        combinator::{alt, delimited},
        token::{take_until, take_while},
    };

    /// Tries to parse a [Token::BlockOpen].
    pub fn block_open<'src>(input: &mut &'src str) -> PResult<Token<'src>> {
        "{".map(|_| Token::BlockOpen).parse_next(input)
    }

    /// Tries to parse a [Token::BlockClose].
    pub fn block_close<'src>(input: &mut &'src str) -> PResult<Token<'src>> {
        "}".map(|_| Token::BlockClose).parse_next(input)
    }

    /// Tries to parse a [Token::Identifier].
    /// Valid identifier characters are `[0-9a-zA-Z_]`.
    /// Does not allow empty identifiers.
    pub fn identifier<'src>(input: &mut &'src str) -> PResult<Token<'src>> {
        take_while(1.., ('0'..='9', 'a'..='z', 'A'..='Z', '_'))
            .map(Token::Identifier)
            .parse_next(input)
    }

    /// Tries to parse a [Token::Separator].
    pub fn separator<'src>(input: &mut &'src str) -> PResult<Token<'src>> {
        ":".map(|_| Token::Separator).parse_next(input)
    }

    /// Tries to parse a [Token::Value].
    /// Valid values are parsed using [value::value].
    pub fn value<'src>(input: &mut &'src str) -> PResult<Token<'src>> {
        value::value.map(Token::Value).parse_next(input)
    }

    /// Tries to parse a single-line [Token::Comment].
    ///
    /// Consumes the newline which ends the comment.
    pub fn comment_single<'src>(input: &mut &'src str) -> PResult<Token<'src>> {
        delimited("//", till_line_ending, line_ending)
            .map(Token::Comment)
            .parse_next(input)
    }

    /// Tries to parse a multi-line [Token::Comment].
    pub fn comment_multi<'src>(input: &mut &'src str) -> PResult<Token<'src>> {
        delimited("/*", take_until(0.., "*/"), "*/")
            .map(Token::Comment)
            .parse_next(input)
    }

    /// Tries to parse a [Token::Comment]
    /// (either [single-][comment_single] or [multi-line][comment_multi]).
    pub fn comment<'src>(input: &mut &'src str) -> PResult<Token<'src>> {
        alt((comment_single, comment_multi)).parse_next(input)
    }

    /// Tries to parse any [Token].
    pub fn token<'src>(input: &mut &'src str) -> PResult<Token<'src>> {
        alt((
            block_open,
            block_close,
            separator,
            comment,
            value,
            identifier,
        ))
        .parse_next(input)
    }
}

/// Lexers to lex individual [Value]s.
///
/// All lexers assume their value starts immediately (i.e., no preceding whitespace)
/// and will not consume any subsequent whitespace.
pub mod value {
    use super::*;
    use winnow::{
        ascii::{digit0, digit1},
        combinator::{alt, opt, preceded},
        stream::AsChar,
        token::take_while,
    };

    /// Tries to parse a [Value::String].
    /// Does not allow escaping (`\"`).
    /// Zero-length strings (`""`) are allowed.
    pub fn string<'src>(input: &mut &'src str) -> PResult<Value<'src>> {
        delimited_by("\"", "\"")
            .map(Value::String)
            .parse_next(input)
    }

    /// Tries to parse a [Value::Regex].
    /// Does not allow escaping (`\$`).
    /// Zero-length regexes (`^$`) are allowed.
    pub fn regex<'src>(input: &mut &'src str) -> PResult<Value<'src>> {
        delimited_by("^", "$").map(Value::Regex).parse_next(input)
    }

    /// Tries to parse a [Value::Number].
    pub fn number<'src>(input: &mut &'src str) -> PResult<Value<'src>> {
        let src = *input;
        (digit1, opt(('.', digit0)))
            .map(|(a, b): (&str, Option<(char, &str)>)| {
                a.len()
                    + match b {
                        Some((b, c)) => b.len() + c.len(),
                        None => 0,
                    }
            })
            .map(|l| &src[..l])
            .map(Value::Number)
            .parse_next(input)
    }

    /// Tries to parse a [Value::Boolean].
    pub fn boolean<'src>(input: &mut &'src str) -> PResult<Value<'src>> {
        alt(("true", "false")).map(Value::Boolean).parse_next(input)
    }

    /// Tries to parse a [Value::Boolean].
    pub fn color<'src>(input: &mut &'src str) -> PResult<Value<'src>> {
        preceded(
            '#',
            alt((
                take_while(8, AsChar::is_hex_digit),
                take_while(6, AsChar::is_hex_digit),
            )),
        )
        .map(Value::Color)
        .parse_next(input)
    }

    /// Tries to parse any [Value].
    pub fn value<'src>(input: &mut &'src str) -> PResult<Value<'src>> {
        alt((string, regex, number, boolean, color)).parse_next(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn simple_example() {
        let input = r#"
        property1: "some string"
        property2: 1.2
        ^regex$ /* comment */ : identifier // other comment
        block {
            named_block "name" {
                prop: #c01032
                42: true
            }
        // }
        }
        "#;

        let expected = vec![
            Token::Identifier("property1"),
            Token::Separator,
            Token::Value(Value::String("some string")),
            Token::Identifier("property2"),
            Token::Separator,
            Token::Value(Value::Number("1.2")),
            Token::Value(Value::Regex("regex")),
            Token::Comment(" comment "),
            Token::Separator,
            Token::Identifier("identifier"),
            Token::Comment(" other comment"),
            Token::Identifier("block"),
            Token::BlockOpen,
            Token::Identifier("named_block"),
            Token::Value(Value::String("name")),
            Token::BlockOpen,
            Token::Identifier("prop"),
            Token::Separator,
            Token::Value(Value::Color("c01032")),
            Token::Value(Value::Number("42")),
            Token::Separator,
            Token::Value(Value::Boolean("true")),
            Token::BlockClose,
            Token::Comment(" }"),
            Token::BlockClose,
        ];

        let lexed = lex(input).expect("Failed to lex");

        assert_eq!(lexed, expected);
    }
}
