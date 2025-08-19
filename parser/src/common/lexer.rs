//! Common lexer items
//!
//! - Primitive [Value]s and [functions to lex them][value]

use winnow::{
    combinator::delimited,
    error::ParserError,
    stream::{Compare, FindSlice, Stream, StreamIsPartial},
    token::take_until,
    Parser,
};

/// A value with a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value<T> {
    /// A string
    String(T),
    /// A regex
    Regex(T),
    /// A number
    Number(T),
    /// A number
    Percentage(T),
    /// A boolean
    Boolean(T),
    /// A color
    Color(T),
}

pub fn delimited_by<
    I: Stream + StreamIsPartial + Compare<&'static str> + FindSlice<&'static str>,
    E: ParserError<I>,
>(
    start: &'static str,
    end: &'static str,
) -> impl Parser<I, <I as Stream>::Slice, E> {
    delimited(start, take_until(0.., end), end)
}

/// Lexers to lex individual [Value]s.
///
/// All lexers assume their value starts immediately (i.e., no preceding whitespace)
/// and will not consume any subsequent whitespace.
pub mod value {
    use super::*;
    use winnow::{
        ascii::{digit0, digit1},
        combinator::{alt, opt, preceded, terminated},
        stream::{AsChar, Compare, FindSlice, SliceLen, Stream, StreamIsPartial},
        token::{one_of, take_while},
        ModalResult, Parser,
    };

    /// Tries to parse a [Value::String].
    /// Does not allow escaping (`\"`).
    /// Zero-length strings (`""`) are allowed.
    pub fn string<I: Stream + StreamIsPartial + Compare<&'static str> + FindSlice<&'static str>>(
        input: &mut I,
    ) -> ModalResult<Value<<I as Stream>::Slice>> {
        delimited_by("\"", "\"")
            .map(Value::String)
            .parse_next(input)
    }

    /// Tries to parse a [Value::Regex].
    /// Does not allow escaping (`\$`).
    /// Zero-length regexes (`^$`) are allowed.
    pub fn regex<I: Stream + StreamIsPartial + Compare<&'static str> + FindSlice<&'static str>>(
        input: &mut I,
    ) -> ModalResult<Value<<I as Stream>::Slice>> {
        delimited_by("^", "$")
            .take()
            .map(Value::Regex)
            .parse_next(input)
    }

    /// Tries to parse a number.
    /// Returns the raw slice.
    pub fn number_raw<I: Stream + StreamIsPartial + Compare<&'static str> + Copy>(
        input: &mut I,
    ) -> ModalResult<<I as Stream>::Slice>
    where
        I::Token: AsChar,
        I::Slice: SliceLen,
    {
        let mut src = *input;
        type R<I> = (
            Option<<I as Stream>::Slice>,
            <I as Stream>::Slice,
            Option<(<I as Stream>::Slice, <I as Stream>::Slice)>,
        );
        (opt("-"), digit1, opt((".", digit0)))
            // get the parsed slice of the input
            // by summing the length of the individual fields
            .map(|(n, a, b): R<I>| {
                n.map(|n| n.slice_len()).unwrap_or(0)
                    + a.slice_len()
                    + b.map(|(b, c)| b.slice_len() + c.slice_len()).unwrap_or(0)
            })
            // and then getting a slice of the input of the specified length
            .map(|l| src.next_slice(l))
            .parse_next(input)
    }

    /// Tries to parse a [Value::Number].
    pub fn number<I: Stream + StreamIsPartial + Compare<&'static str> + Copy>(
        input: &mut I,
    ) -> ModalResult<Value<<I as Stream>::Slice>>
    where
        I::Token: AsChar,
        I::Slice: SliceLen,
    {
        number_raw.map(Value::Number).parse_next(input)
    }

    /// Tries to parse a [Value::Percentage].
    pub fn percentage<I: Stream + StreamIsPartial + Compare<&'static str> + Copy>(
        input: &mut I,
    ) -> ModalResult<Value<<I as Stream>::Slice>>
    where
        I::Token: AsChar + Clone,
        I::Slice: SliceLen,
    {
        terminated(number_raw, one_of(['%']))
            .map(Value::Percentage)
            .parse_next(input)
    }

    /// Tries to parse a [Value::Boolean].
    pub fn boolean<I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> ModalResult<Value<<I as Stream>::Slice>> {
        alt(("true", "false")).map(Value::Boolean).parse_next(input)
    }

    /// Tries to parse a [Value::Boolean].
    pub fn color<I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> ModalResult<Value<I::Slice>>
    where
        I::Token: AsChar + Clone,
    {
        preceded(
            "#",
            alt((
                take_while(8, AsChar::is_hex_digit),
                take_while(6, AsChar::is_hex_digit),
            )),
        )
        .map(Value::Color)
        .parse_next(input)
    }

    /// Tries to parse any [Value].
    pub fn value<
        I: Stream + StreamIsPartial + Compare<&'static str> + FindSlice<&'static str> + Copy,
    >(
        input: &mut I,
    ) -> ModalResult<Value<<I as Stream>::Slice>>
    where
        I::Token: AsChar + Clone,
        I::Slice: SliceLen,
    {
        alt((string, regex, percentage, number, boolean, color)).parse_next(input)
    }
}

/// A generic token (that is common to all formats)
/// Derived Tokens should be able to convert from [GenericToken] and to [`Option<GenericToken>`]
pub enum GenericToken<T> {
    /// An identifier
    Identifier(T),
    /// A value; see [Value]
    Value(Value<T>),
    /// A comment, either single- or multiline
    Comment(T),
    /// Opening-symbol for a tuple
    TupleOpen,
    /// Closing-symbol for a tuple
    TupleClose,
    // Opening-symbol for a set of values
    SetOpen,
    // Closing-symbol for a set of values
    SetClose,
    /// A separator between elements (e.g., in tuples)
    ElementSeparator,
}

/// Functions to parse [GenericToken]s.
/// These will automatically try to convert the [GenericToken] into a token,
/// assuming the token can be converted using [Into::into].
pub mod token {
    use super::*;
    use winnow::{
        ascii::{line_ending, till_line_ending},
        combinator::{alt, delimited},
        stream::{AsChar, Compare, FindSlice, SliceLen, Stream, StreamIsPartial},
        token::{take_until, take_while},
        ModalResult, Parser,
    };

    /// Tries to parse a [GenericToken::Identifier].
    /// Valid identifier characters are `[0-9a-zA-Z_]`.
    /// Does not allow empty identifiers.
    pub fn identifier<Tok, I: Stream + StreamIsPartial>(input: &mut I) -> ModalResult<Tok>
    where
        <I as Stream>::Token: AsChar + Clone,
        GenericToken<I::Slice>: Into<Tok>,
    {
        take_while(1.., ('0'..='9', 'a'..='z', 'A'..='Z', '_'))
            .map(GenericToken::Identifier)
            .output_into()
            .parse_next(input)
    }

    /// Tries to parse a [GenericToken::Value].
    /// Valid values are parsed using [value::value].
    pub fn value<
        Tok,
        I: Stream + StreamIsPartial + Compare<&'static str> + FindSlice<&'static str> + Copy,
    >(
        input: &mut I,
    ) -> ModalResult<Tok>
    where
        I::Token: AsChar + Clone,
        I::Slice: SliceLen,
        GenericToken<I::Slice>: Into<Tok>,
    {
        value::value
            .map(GenericToken::Value)
            .output_into()
            .parse_next(input)
    }

    /// Tries to parse a single-line [GenericToken::Comment].
    ///
    /// Consumes the newline which ends the comment.
    pub fn comment_single<
        Tok,
        I: Stream + StreamIsPartial + Compare<&'static str> + FindSlice<(char, char)>,
    >(
        input: &mut I,
    ) -> ModalResult<Tok>
    where
        <I as Stream>::Token: AsChar + Clone,
        GenericToken<I::Slice>: Into<Tok>,
    {
        delimited("//", till_line_ending, line_ending)
            .map(GenericToken::Comment)
            .output_into()
            .parse_next(input)
    }

    /// Tries to parse a multi-line [GenericToken::Comment].
    pub fn comment_multi<
        Tok,
        I: Stream + StreamIsPartial + Compare<&'static str> + FindSlice<&'static str>,
    >(
        input: &mut I,
    ) -> ModalResult<Tok>
    where
        GenericToken<I::Slice>: Into<Tok>,
    {
        delimited("/*", take_until(0.., "*/"), "*/")
            .map(GenericToken::Comment)
            .output_into()
            .parse_next(input)
    }

    /// Tries to parse a [GenericToken::Comment]
    /// (either [single-][comment_single] or [multi-line][comment_multi]).
    pub fn comment<
        Tok,
        I: Stream
            + StreamIsPartial
            + Compare<&'static str>
            + FindSlice<&'static str>
            + FindSlice<(char, char)>,
    >(
        input: &mut I,
    ) -> ModalResult<Tok>
    where
        <I as Stream>::Token: AsChar + Clone,
        GenericToken<I::Slice>: Into<Tok>,
    {
        alt((comment_single, comment_multi)).parse_next(input)
    }

    /// Tries to parse a [GenericToken::TupleOpen].
    pub fn tuple_open<Tok, I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> ModalResult<Tok>
    where
        GenericToken<I::Slice>: Into<Tok>,
    {
        "(".map(|_| GenericToken::TupleOpen)
            .output_into()
            .parse_next(input)
    }

    /// Tries to parse a [GenericToken::TupleClose].
    pub fn tuple_close<Tok, I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> ModalResult<Tok>
    where
        GenericToken<I::Slice>: Into<Tok>,
    {
        ")".map(|_| GenericToken::TupleClose)
            .output_into()
            .parse_next(input)
    }

    /// Tries to parse a [GenericToken::SetOpen].
    pub fn set_open<Tok, I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> ModalResult<Tok>
    where
        GenericToken<I::Slice>: Into<Tok>,
    {
        "{".map(|_| GenericToken::SetOpen)
            .output_into()
            .parse_next(input)
    }

    /// Tries to parse a [GenericToken::SetClose].
    pub fn set_close<Tok, I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> ModalResult<Tok>
    where
        GenericToken<I::Slice>: Into<Tok>,
    {
        "}".map(|_| GenericToken::SetClose)
            .output_into()
            .parse_next(input)
    }

    /// Tries to parse a [GenericToken::ElementSeparator].
    pub fn element_separator<Tok, I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> ModalResult<Tok>
    where
        GenericToken<I::Slice>: Into<Tok>,
    {
        ",".map(|_| GenericToken::ElementSeparator)
            .output_into()
            .parse_next(input)
    }
}
