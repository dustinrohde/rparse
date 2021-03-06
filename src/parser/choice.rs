use crate::error::Expected;
use {ParseResult, Parser, Stream};

pub struct Skip<P1, P2> {
    p1: P1,
    p2: P2,
}

impl<S, P1, P2> Parser for Skip<P1, P2>
where
    S: Stream,
    P1: Parser<Stream = S>,
    P2: Parser<Stream = S>,
{
    type Stream = S;
    type Output = P1::Output;

    fn parse_partial(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        let (result, stream) = self.p1.parse_partial(stream)?;
        let (_, stream) = self.p2.parse_partial(stream)?;
        stream.result(result)
    }
}

/// Parses with `p1` followed by `p2`. Succeeds if both parsers succeed, otherwise fails.
/// Returns the result of `p1` on success.
///
/// Useful when you need a parser to consume input but you don't care about the result. Equivalent
/// to [`p1.skip(p2)`].
///
/// [`p1.skip(p2)`]: Parser::skip
pub fn skip<P1, P2>(p1: P1, p2: P2) -> Skip<P1, P2> {
    Skip { p1, p2 }
}

pub struct With<P1, P2> {
    p1: P1,
    p2: P2,
}

impl<S, P1, P2> Parser for With<P1, P2>
where
    S: Stream,
    P1: Parser<Stream = S>,
    P2: Parser<Stream = S>,
{
    type Stream = S;
    type Output = P2::Output;

    fn parse_partial(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        let (_, stream) = self.p1.parse_partial(stream)?;
        let (result, stream) = self.p2.parse_partial(stream)?;
        stream.result(result)
    }
}

/// Parses with `p1` followed by `p2`. Succeeds if both parsers succeed, otherwise fails.
/// Returns the result of `p2` on success.
///
/// Useful when you need a parser to consume input but you don't care about the result. Equivalent
/// to [`p1.with(p2)`].
///
/// [`p1.with(p2)`]: Parser::with
pub fn with<P1, P2>(p1: P1, p2: P2) -> With<P1, P2> {
    With { p1, p2 }
}

pub struct Optional<P> {
    p: P,
}

impl<P> Parser for Optional<P>
where
    P: Parser,
{
    type Stream = P::Stream;
    type Output = P::Output;

    fn parse_lazy(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        let backup = stream.backup();
        let mut stream = match self.p.parse_lazy(stream) {
            Ok((Some(result), stream)) => return stream.ok(result),
            Ok((None, stream)) => stream,
            Err((_, stream)) => stream,
        };
        stream.restore(backup);
        stream.result(None)
    }

    fn expected_error(&self) -> Option<Expected<Self::Stream>> {
        self.p.expected_error()
    }
}

/// Wrap `p` so that if it would fail it returns `None` instead. Equivalent to
/// [`parser.optional()`].
///
/// [`parser.optional()`]: Parser::optional
pub fn optional<P: Parser>(p: P) -> Optional<P> {
    Optional { p }
}

pub struct Must<P> {
    p: P,
}

impl<P> Parser for Must<P>
where
    P: Parser,
{
    type Stream = P::Stream;
    type Output = P::Output;

    fn parse_lazy(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        match self.p.parse_lazy(stream) {
            Ok((Some(result), stream)) => stream.ok(result),
            Ok((None, stream)) => {
                let mut error = stream.new_error();
                self.add_expected_error(&mut error);
                stream.err(error)
            }
            Err((err, stream)) => stream.err(err),
        }
    }

    fn expected_error(&self) -> Option<Expected<Self::Stream>> {
        self.p.expected_error()
    }
}

pub fn must<P: Parser>(p: P) -> Must<P> {
    Must { p }
}

pub struct Or<L, R> {
    p1: L,
    p2: R,
}

impl<S: Stream, O, L, R> Parser for Or<L, R>
where
    L: Parser<Stream = S, Output = O>,
    R: Parser<Stream = S, Output = O>,
{
    type Stream = S;
    type Output = O;

    fn parse_lazy(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        match self.p1.try_parse_lazy(stream) {
            Ok((result, stream)) => stream.result(result),
            Err((_, stream)) => self.p2.parse_lazy(stream),
        }
    }

    fn expected_error(&self) -> Option<Expected<Self::Stream>> {
        Expected::merge_one_of(vec![self.p1.expected_error(), self.p2.expected_error()])
    }
}

/// Equivalent to [`p1.or(p2)`].
///
/// [`p1.or(p2)`]: Parser::or
pub fn or<S: Stream, O, L, R>(p1: L, p2: R) -> Or<L, R>
where
    L: Parser<Stream = S, Output = O>,
    R: Parser<Stream = S, Output = O>,
{
    Or { p1, p2 }
}

/// Try one or more parsers, returning from the first one that succeeds.
///
/// Equivalent to chaining parsers with [`or`]. For example, these two parsers are equivalent:
///
/// ```
/// # #[macro_use]
/// # extern crate rparse;
/// # use rparse::Parser;
/// # use rparse::parser::item::item;
/// # fn main() {
/// let mut p1 = choice![item(b'x'), item(b'y'), item(b'z')];
/// let mut p2 = item(b'x').or(item(b'y').or(item(b'z')));
/// assert_eq!(p1.parse("x123"), p2.parse("x123"));
/// # }
/// ```
///
/// [`or`]: Parser::or
#[macro_export]
macro_rules! choice {
    ($head:expr) => {
        $head
    };
    ($head:expr, $($tail:expr),+ $(,)*) => {
        $head.or(choice!($($tail),+))
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use error::{Error, Info};
    use parser::{
        item::{any, ascii, item},
        range::range,
        repeat::{many, many1},
        seq::then,
        test_utils::*,
    };
    use stream::IndexedStream;

    #[test]
    fn test_optional() {
        let mut parser = optional(item(b'x'));
        test_parser!(&str => char | parser, {
            "" => noop(),
            "y" => noop(),
            "x" => ok('x', ""),
            "xyz" => ok('x', "yz"),
        });

        let mut parser = optional(item(b'x'));
        test_parser!(&str => char | parser, {
            "" => noop(),
            "y" => noop(),
            "x" => ok('x', ""),
            "xyz" => ok('x', "yz"),
        });

        let mut parser = optional(many1(ascii::alpha_num()));
        test_parser!(&str => Vec<char> | parser, {
            "abc123" => ok("abc123".chars().collect(), ""),
        });

        let mut parser = optional(many(any()));
        assert_eq!(parser.parse(""), ok_result(vec![], ""));

        let mut parser = item(b'x').optional();
        test_parser!(&str => char | parser, {
            "x" => ok('x', ""),
            "y" => noop(),
        });
    }

    #[test]
    fn test_with() {
        let mut parser = with(item(b'a'), item(b'b'));
        test_parser!(IndexedStream<&str> => char | parser, {
            "abcd" => ok('b', ("cd", 2)),
            "ab" => ok('b', ("", 2)),
            "def" => err(Error::item('d').expected(b'a').at(0)),
            "aab" => err(Error::item('a').expected(b'b').at(1)),
            "bcd" => err(Error::item('b').expected(b'a').at(0)),
        });

        let mut parser = with(many1::<Vec<_>, _>(ascii::digit()), many1(ascii::letter()));
        test_parser!(IndexedStream<&str> => Vec<char> | parser, {
            "123abc456" => ok(vec!['a', 'b', 'c'], ("456", 6)),
            " 1 2 3" => err(Error::item(' ').expected("an ascii digit").at(0)),
            "123 abc" => err(Error::item(' ').expected("an ascii letter").at(3)),
        });
    }

    #[test]
    fn test_or() {
        let mut parser = or(item(b'a'), item(b'b'));
        test_parser!(IndexedStream<&str> => char | parser, {
            "bcd" => ok('b', ("cd", 1)),
            "a" => ok('a', ("", 1)),
            "def" => err(Error::item('d').expected_one_of(vec![b'a', b'b']).at(0)),
        });

        let mut parser = or(
            many1::<String, _>(ascii::digit()),
            then(ascii::letter(), ascii::whitespace()),
        );
        test_parser!(IndexedStream<&str> => String | parser, {
            "123a bc" => ok("123".into(), ("a bc", 3)),
            "a b c" => ok("a ".into(), ("b c", 2)),
        });

        let mut parser = or(or(range("feel"), range("feet")), range("fee"));
        test_parser!(IndexedStream<&str> => &str | parser, {
            "feel" => ok("feel", ("", 4)),
            "feet" => ok("feet", ("", 4)),
            "fees" => ok("fee", ("s", 3)),
            "fern" => err(Error::item('r').at(2).expected_one_of(vec![
                Info::Range("feel"),
                Info::Range("feet"),
                Info::Range("fee"),
            ])),
        });
    }

    #[test]
    fn test_choice() {
        assert_eq!(
            choice!(item(b'a'), item(b'b')).parse("a"),
            ok_result('a', "")
        );

        let mut parser = choice!(item(b'a'), ascii::digit(), ascii::punctuation());
        test_parser!(IndexedStream<&str> => char | parser, {
            "a9." => ok('a', ("9.", 1)),
            "9.a" => ok('9', (".a", 1)),
            ".a9" => ok('.', ("a9", 1)),
            "ba9." => err(
                Error::item('b')
                    .expected_one_of(vec![
                        Info::Item('a'),
                        Info::Msg("an ascii digit"),
                        Info::Msg("an ascii punctuation character"),
                    ])
                    .at(0)
            ),
        });

        assert_eq!(
            choice!(item(b'a'), item(b'b'), item(b'c')).parse("bcd"),
            ok_result('b', "cd"),
        );

        assert_eq!(choice!(ascii::letter()).parse("Z"), ok_result('Z', ""));

        let mut parser = choice![
            many1::<String, _>(ascii::digit()),
            then(ascii::letter(), ascii::whitespace()),
        ];
        test_parser!(IndexedStream<&str> => String | parser, {
            "123a bc" => ok("123".to_string(), ("a bc", 3)),
            "a b c" => ok("a ".to_string(), ("b c", 2)),
        });
    }
}
