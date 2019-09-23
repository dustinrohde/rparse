//! Parsers that transform the output of other Parsers with arbitrary functions.

use std::fmt::Display;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::option::Option::*;
use std::str;

use error::Error;
use traits::StrLike;
use {ParseResult, Parser, Stream};

pub struct Expect<P: Parser> {
    parser: P,
    error: Error<P::Stream>,
}

impl<P: Parser> Expect<P> {
    pub fn expected_error(&self) -> Error<P::Stream> {
        self.error.clone()
    }
}

impl<P: Parser> Parser for Expect<P> {
    type Stream = P::Stream;
    type Output = P::Output;

    fn parse_lazy(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        self.parser.parse_lazy(stream)
    }

    fn expected_errors(&self) -> Vec<Error<Self::Stream>> {
        vec![self.error.clone()]
    }
}

pub fn expect<P, I>(parser: P, expected: I) -> Expect<P>
where
    P: Parser,
    I: Into<Error<P::Stream>>,
{
    Expect {
        parser,
        error: Error::expected(expected),
    }
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
        match self.parser.parse_lazy(stream) {
            Ok((result, stream)) => stream.result(result.map(|output| (self.f)(output))),
            Err((err, stream)) => stream.errs(err),
        }
    }

    fn expected_errors(&self) -> Vec<Error<Self::Stream>> {
        self.parser.expected_errors()
    }
}

pub fn map<P, F, O>(p: P, f: F) -> Map<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> O,
{
    Map { parser: p, f }
}

pub type Iter<P, I> = Map<P, fn(<P as Parser>::Output) -> <I as IntoIterator>::IntoIter>;

pub fn iter<P, I>(p: P) -> Iter<P, I>
where
    P: Parser<Output = I>,
    I: IntoIterator,
{
    p.map(|output| output.into_iter())
}

pub type Collect<P, O> = Map<P, fn(<P as Parser>::Output) -> O>;

pub fn collect<P, O>(p: P) -> Collect<P, O>
where
    P: Parser,
    P::Output: IntoIterator,
    O: FromIterator<<P::Output as IntoIterator>::Item>,
{
    p.map(|output| output.into_iter().collect())
}

pub type Flatten<P, O> = Map<P, fn(<P as Parser>::Output) -> Vec<O>>;

pub fn flatten<P, O>(p: P) -> Flatten<P, O>
where
    P: Parser<Output = Vec<Vec<O>>>,
{
    p.map(|output| output.into_iter().flatten().collect())
}

pub type Wrap<P> = Map<P, fn(<P as Parser>::Output) -> Vec<<P as Parser>::Output>>;

pub fn wrap<P>(p: P) -> Wrap<P>
where
    P: Parser,
{
    p.map(|output| {
        let mut v = Vec::new();
        v.push(output);
        v
    })
}

pub struct Bind<P, F> {
    parser: P,
    f: F,
}

impl<P, F, O> Parser for Bind<P, F>
where
    P: Parser,
    F: Fn(Option<P::Output>, P::Stream) -> ParseResult<P::Stream, O>,
{
    type Stream = P::Stream;
    type Output = O;

    fn parse_lazy(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        match self.parser.parse_lazy(stream) {
            Ok((result, stream)) => (self.f)(result, stream),
            Err((err, stream)) => stream.errs(err),
        }
    }

    fn expected_errors(&self) -> Vec<Error<Self::Stream>> {
        self.parser.expected_errors()
    }
}

pub fn bind<P, F, O>(p: P, f: F) -> Bind<P, F>
where
    P: Parser,
    F: Fn(Option<P::Output>, P::Stream) -> O,
{
    Bind { parser: p, f }
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
        match self.parser.parse_partial(stream) {
            Ok((Some(s), stream)) => {
                let result = s
                    .from_utf8()
                    .map_err(|_| "invalid UTF-8".into())
                    .and_then(|s: &str| s.parse::<O>().map_err(|e: O::Err| e.to_string().into()));
                match result {
                    Ok(output) => stream.ok(output),
                    Err(err) => stream.err(err),
                }
            }
            Ok((None, stream)) => stream.noop(),
            Err((err, stream)) => stream.errs(err),
        }
    }
}

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
    use error::Error::*;
    use parser::seq::many1;
    use parser::test_utils::*;
    use parser::token::{ascii, token};
    use stream::{IndexedStream, SourceCode};

    #[test]
    fn test_map() {
        let mut parser = map(ascii::digit(), |c: char| c.to_string());
        test_parser!(&str => String | parser, {
            "3" => ok("3".to_string(), ""),
            "a3" => err(vec![Unexpected('a'.into()), Error::expected("an ascii digit")]),
        });

        let mut parser = map(many1(ascii::letter()).collect::<String>(), |s| {
            s.to_uppercase()
        });
        assert_eq!(
            parser.parse("aBcD12e"),
            ok_result("ABCD".to_string(), "12e")
        );

        let mut parser = map(many1(ascii::alpha_num()).collect::<String>(), |s| {
            s.parse::<usize>().unwrap_or(0)
        });
        test_parser!(&str => usize | parser, {
            "324 dogs" => ok(324 as usize, " dogs"),
            "324dogs" => ok(0 as usize, ""),
        });
    }

    #[test]
    fn test_bind() {
        // TODO: use realistic use cases for these tests. many of these are better suited to map()
        let mut parser = ascii::digit()
            .bind(|r: Option<char>, stream: &str| stream.result(r.map(|c| c.to_string())));
        test_parser!(&str => String | parser, {
            "3" => ok("3".to_string(), ""),
            "a3" => err(vec![Error::unexpected_token('a'), Error::expected("an ascii digit")]),
        });

        let mut parser = many1(ascii::letter())
            .collect()
            .bind(|r: Option<String>, stream: &str| stream.result(r.map(|s| s.to_uppercase())));
        assert_eq!(
            parser.parse("aBcD12e"),
            ok_result("ABCD".to_string(), "12e")
        );

        let mut parser = many1(ascii::alpha_num()).collect().bind(
            |r: Option<String>, stream: IndexedStream<&str>| match r {
                Some(s) => match s.parse::<usize>() {
                    Ok(n) => stream.ok(n),
                    Err(e) => stream.err(Box::new(e).into()),
                },
                _ => stream.noop(),
            },
        );
        test_parser!(IndexedStream<&str> => usize | parser, {
            "324 dogs" => ok(324 as usize, (" dogs", 3)),
        // TODO: add ability to control consumption, e.g. make this error show at beginning (0)
        // TODO: e.g.: many1(alpha_num()).bind(...).try()
            "324dogs" => err(7, vec!["invalid digit found in string".into(), Error::expected("an ascii letter or digit")]),
        });
    }

    #[test]
    fn test_from_str() {
        let mut parser = many1(ascii::digit()).collect::<String>().from_str::<u32>();
        test_parser!(&str => u32 | parser, {
            "369" => ok(369 as u32, ""),
            "369abc" => ok(369 as u32, "abc"),
            "abc" => err(vec![Unexpected('a'.into()), Error::expected("an ascii digit")]),
        });

        let mut parser = many1(choice!(token(b'-'), token(b'.'), ascii::digit()))
            .collect::<String>()
            .from_str::<f32>();
        test_parser!(&str => f32 | parser, {
            "12e" => ok(12 as f32, "e"),
            "-12e" => ok(-12 as f32, "e"),
            "-12.5e" => ok(-12.5 as f32, "e"),
            "12.5.9" =>  err(vec!["invalid float literal".into()]),
        });

        let mut parser = many1(ascii::digit()).collect::<String>().from_str::<f32>();
        test_parser!(SourceCode => f32 | parser, {
            "12e" => ok(12f32, ("e", (1, 3))),
            "e12" => err((1, 1), vec![Unexpected('e'.into()), Error::expected("an ascii digit")]),
        });
    }
}
