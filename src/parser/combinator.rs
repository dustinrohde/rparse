//! Parsers that transform other Parsers.

use std::fmt::Display;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::option::Option::*;
use std::str;

use crate::{Expected, ParseResult, Parser, Stream};
use traits::StrLike;

pub struct Expect<P: Parser> {
    parser: P,
    expected: Option<Expected<P::Stream>>,
}

impl<P: Parser> Parser for Expect<P> {
    type Stream = P::Stream;
    type Output = P::Output;

    fn parse_lazy(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        self.parser.parse_lazy(stream)
    }

    fn expected_error(&self) -> Option<Expected<Self::Stream>> {
        self.expected.clone()
    }
}

/// Equivalent to [`parser.expect(error)`](Parser::expect).
pub fn expect<P, I>(parser: P, expected: I) -> Expect<P>
where
    P: Parser,
    I: Into<Expected<P::Stream>>,
{
    Expect {
        parser,
        expected: Some(expected.into()),
    }
}

/// Equivalent to [`parser.no_expect(error)`](Parser::no_expect).
pub fn no_expect<P>(parser: P) -> Expect<P>
where
    P: Parser,
{
    Expect {
        parser,
        expected: None,
    }
}

pub struct Attempt<P: Parser> {
    p: P,
}

impl<P: Parser> Parser for Attempt<P> {
    type Stream = P::Stream;
    type Output = P::Output;

    fn parse_lazy(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        let backup = stream.backup();
        let mut result = self.p.parse_lazy(stream);
        if let Err((_, ref mut stream)) = result {
            stream.restore(backup);
        }
        result
    }

    fn try_parse_lazy(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        self.parse_lazy(stream)
    }

    fn expected_error(&self) -> Option<Expected<Self::Stream>> {
        self.p.expected_error()
    }
}

pub fn attempt<P: Parser>(p: P) -> Attempt<P> {
    Attempt { p }
}

pub struct Lookahead<P: Parser> {
    p: P,
}

impl<P: Parser> Parser for Lookahead<P> {
    type Stream = P::Stream;
    type Output = P::Output;

    fn parse_lazy(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        let start = stream.backup();
        match self.p.parse_lazy(stream) {
            Ok((result, _)) => start.result(result),
            Err((error, _)) => start.err(error),
        }
    }

    fn try_parse_lazy(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        self.parse_lazy(stream)
    }

    fn parse(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        self.parse_partial(stream)
    }
}

pub fn lookahead<P: Parser>(p: P) -> Lookahead<P> {
    Lookahead { p }
}

pub struct Map<P, F> {
    parser: P,
    f: F,
}

impl<P, F, O> Parser for Map<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> O,
{
    type Stream = P::Stream;
    type Output = O;

    fn parse_lazy(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        let (result, stream) = self.parser.parse_lazy(stream)?;
        stream.result(result.map(|output| (self.f)(output)))
    }

    fn expected_error(&self) -> Option<Expected<Self::Stream>> {
        self.parser.expected_error()
    }
}

/// Equivalent to [`parser.map(f)`].
///
/// [`parser.map(f)`]: Parser::map
pub fn map<P, F, O>(parser: P, f: F) -> Map<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> O,
{
    Map { parser, f }
}

pub type Collect<P, O> = Map<P, fn(<P as Parser>::Output) -> O>;

/// Equivalent to [`p.collect()`].
///
/// [`p.collect()`]: Parser::collect
pub fn collect<P, O>(p: P) -> Collect<P, O>
where
    P: Parser,
    P::Output: IntoIterator,
    O: FromIterator<<P::Output as IntoIterator>::Item>,
{
    p.map(|output| output.into_iter().collect())
}

pub type Flatten<O, P> = Map<P, fn(<P as Parser>::Output) -> O>;

/// Equivalent to [`p.flatten()`].
///
/// [`p.flatten()`]: Parser::flatten
pub fn flatten<O, P>(p: P) -> Flatten<O, P>
where
    P: Parser,
    P::Output: IntoIterator,
    <P::Output as IntoIterator>::Item: IntoIterator,
    O: std::iter::Extend<<<P::Output as IntoIterator>::Item as IntoIterator>::Item> + Default,
{
    p.map(|output| {
        output
            .into_iter()
            .fold(O::default(), |mut acc, collection| {
                acc.extend(collection.into_iter());
                acc
            })
    })
}

pub type Wrap<O, P> = Map<P, fn(<P as Parser>::Output) -> O>;

/// Equivalent to [`p.wrap()`].
///
/// [`p.wrap()`]: Parser::wrap
pub fn wrap<O, P>(p: P) -> Wrap<O, P>
where
    P: Parser,
    O: Extend<P::Output> + Default,
{
    p.map(|output| {
        let mut wrapped = O::default();
        wrapped.extend(std::iter::once(output));
        wrapped
    })
}

pub struct AndThen<P, F> {
    parser: P,
    f: F,
}

impl<P, F, O> Parser for AndThen<P, F>
where
    P: Parser,
    F: Fn(P::Output, P::Stream) -> ParseResult<P::Stream, O>,
{
    type Stream = P::Stream;
    type Output = O;

    fn parse_lazy(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        let (result, stream) = self.parser.parse_lazy(stream)?;
        match result {
            Some(value) => (self.f)(value, stream),
            None => stream.noop(),
        }
    }

    fn expected_error(&self) -> Option<Expected<Self::Stream>> {
        self.parser.expected_error()
    }
}

/// Equivalent to [`p.and_then()`].
///
/// [`p.and_then()`]: Parser::and_then
pub fn and_then<P, F, O>(p: P, f: F) -> AndThen<P, F>
where
    P: Parser,
    F: Fn(P::Output, P::Stream) -> O,
{
    AndThen { parser: p, f }
}

pub struct FromStr<P, O> {
    parser: P,
    _marker: PhantomData<O>,
}

impl<P, O> Parser for FromStr<P, O>
where
    P: Parser,
    P::Output: StrLike,
    O: str::FromStr,
    O::Err: Display,
{
    type Stream = P::Stream;
    type Output = O;

    fn parse_partial(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        match self.parser.parse_partial(stream)? {
            (Some(s), stream) => {
                let result = s
                    .from_utf8()
                    .map_err(|_| "invalid UTF-8".into())
                    .and_then(|s: &str| s.parse::<O>().map_err(|e: O::Err| e.to_string().into()));
                match result {
                    Ok(output) => stream.ok(output),
                    Err(err) => stream.err(err),
                }
            }
            (None, stream) => stream.noop(),
        }
    }
}

/// Equivalent to [`p.from_str()`].
///
/// [`p.from_str()`]: Parser::from_str
pub fn from_str<P, O>(parser: P) -> FromStr<P, O>
where
    P: Parser,
    P::Output: StrLike,
    O: str::FromStr,
    O::Err: Display,
{
    FromStr {
        parser,
        _marker: PhantomData,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use error::Error;
    use parser::{
        item::{ascii, item},
        range::range,
        repeat::many1,
        seq::then,
        test_utils::*,
    };
    use stream::{IndexedStream, SourceCode};

    #[test]
    fn test_attempt() {
        let mut parser = attempt(range("abcdef"));
        let (result, stream) = parser.parse_lazy(IndexedStream::from("abcdef")).unwrap();
        assert_eq!(result, Some("abcdef"));
        assert_eq!(stream, ("", 6).into());

        let (_, stream) = range("abcdef")
            .parse_lazy(IndexedStream::from("abcd!!!!"))
            .unwrap_err();
        assert_eq!(stream, ("!!", 6).into());
        let (_, stream) = parser
            .parse_lazy(IndexedStream::from("abcde!!!"))
            .unwrap_err();
        assert_eq!(stream, ("abcde!!!", 0).into());
    }

    #[test]
    fn test_map() {
        let mut parser = map(ascii::digit(), |c: char| c.to_string());
        test_parser!(&str => String | parser, {
            "3" => ok("3".to_string(), ""),
            "" => err(Error::eoi().expected("an ascii digit")),
            "a3" => err(Error::item('a').expected("an ascii digit")),
        });

        let mut parser = map(many1::<String, _>(ascii::letter()), |s| s.to_uppercase());
        assert_eq!(
            parser.parse("aBcD12e"),
            ok_result("ABCD".to_string(), "12e")
        );

        let mut parser = map(many1::<String, _>(ascii::alpha_num()), |s| {
            s.parse::<usize>().unwrap_or(0)
        });
        test_parser!(&str => usize | parser, {
            "324 dogs" => ok(324 as usize, " dogs"),
            "324dogs" => ok(0 as usize, ""),
        });
    }

    #[test]
    fn test_and_then() {
        // TODO: use realistic use cases for these tests. many of these are better suited to map()
        let mut parser = ascii::digit().and_then(|c: char, stream: &str| stream.ok(c.to_string()));
        test_parser!(&str => String | parser, {
            "3" => ok("3".to_string(), ""),
            "a3" => err(Error::item('a').expected("an ascii digit")),
        });

        let mut parser = many1::<String, _>(ascii::letter())
            .and_then(|s: String, stream: &str| stream.ok(s.to_uppercase()));
        assert_eq!(
            parser.parse("aBcD12e"),
            ok_result("ABCD".to_string(), "12e")
        );

        let mut parser = many1::<String, _>(ascii::alpha_num()).and_then(
            |s: String, stream: IndexedStream<&str>| match s.parse::<usize>() {
                Ok(n) => stream.ok(n),
                Err(e) => stream.err(Box::new(e).into()),
            },
        );
        test_parser!(IndexedStream<&str> => usize | parser, {
            "324 dogs" => ok(324 as usize, (" dogs", 3)),
            // TODO: add ability to control consumption, e.g. make this error show at beginning (0)
            // TODO: e.g.: many1(alpha_num()).and_then(...).try()
            "324dogs" => err(
                Error::from("invalid digit found in string")
                    .expected("an ascii letter or digit")
                    .at(7)
            ),
        });
    }

    #[test]
    fn test_collect() {
        let mut parser = collect(many1::<Vec<_>, _>(ascii::digit()));
        test_parser!(IndexedStream<&str> => String | parser, {
            "123" => ok("123".to_string(), ("", 3)),
            "123abc" => ok("123".to_string(), ("abc", 3)),
            "" => err(Error::eoi().expected("an ascii digit").at(0)),
            "abc" => err(Error::item('a').expected("an ascii digit").at(0)),
        });
    }

    #[test]
    fn test_flatten() {
        let mut parser = flatten(then::<Vec<Vec<_>>, _, _>(
            many1(ascii::digit()),
            many1(ascii::letter()),
        ));
        test_parser!(IndexedStream<&str> => Vec<char> | parser, {
            "1a" => ok(vec!['1', 'a'], ("", 2)),
            "0bb3" => ok(vec!['0', 'b', 'b'], ("3", 3)),
            "" => err(Error::eoi().expected("an ascii digit").at(0)),
            "3\t" => err(Error::item('\t').expected("an ascii letter").at(1)),
        });
    }

    #[test]
    fn test_wrap() {
        let mut parser = item(b'x').wrap();
        test_parser!(IndexedStream<&str> => Vec<char> | parser, {
            "x" => ok(vec!['x'], ("", 1)),
            "" => err(Error::eoi().expected(b'x').at(0)),
            "\t" => err(Error::item('\t').expected(b'x').at(0)),
        });
    }

    #[test]
    fn test_from_str() {
        let mut parser = many1::<String, _>(ascii::digit()).from_str::<u32>();
        test_parser!(&str => u32 | parser, {
            "369" => ok(369 as u32, ""),
            "369abc" => ok(369 as u32, "abc"),
            "abc" => err(Error::item('a').expected("an ascii digit")),
        });

        let mut parser =
            many1::<String, _>(choice!(item(b'-'), item(b'.'), ascii::digit())).from_str::<f32>();
        test_parser!(&str => f32 | parser, {
            "12e" => ok(12 as f32, "e"),
            "-12e" => ok(-12 as f32, "e"),
            "-12.5e" => ok(-12.5 as f32, "e"),
            "12.5.9" =>  err(Error::from("invalid float literal")),
        });

        let mut parser = many1::<String, _>(ascii::digit()).from_str::<f32>();
        test_parser!(SourceCode => f32 | parser, {
            "12e" => ok(12f32, ("e", (1, 3))),
            "e12" => err(Error::item('e').expected("an ascii digit").at((1, 1))),
        });
    }
}
