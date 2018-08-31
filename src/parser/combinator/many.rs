use std::iter::FromIterator;
use std::marker::PhantomData;

use error::ParseResult;
use parser::Parser;
use stream::Stream;

pub struct Many<P, O> {
    p: P,
    min: usize,
    __marker: PhantomData<O>,
}

impl<P, O> Parser for Many<P, O>
where
    P: Parser,
    O: FromIterator<P::Output>,
{
    type Stream = P::Stream;
    type Output = O;

    fn parse_stream(
        &mut self,
        mut stream: Self::Stream,
    ) -> ParseResult<Self::Stream, Self::Output> {
        let mut output = Vec::new();
        let mut i = 0;
        loop {
            match self.p.parse(stream) {
                (Ok(result), rest) => {
                    output.push(result);
                    stream = rest;
                }
                (Err(err), rest) => {
                    if i < self.min {
                        break rest.errs(err);
                    }
                    stream = rest;
                    break stream.ok(output.into_iter().collect());
                }
            }

            i += 1;
        }
    }
}

pub fn many<P, O>(p: P) -> Many<P, O>
where
    P: Parser,
    O: FromIterator<P::Output>,
{
    Many {
        p,
        min: 0,
        __marker: PhantomData,
    }
}

pub fn many1<P, O>(p: P) -> Many<P, O>
where
    P: Parser,
    O: FromIterator<P::Output>,
{
    Many {
        p,
        min: 1,
        __marker: PhantomData,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::token::token;

    #[test]
    fn test_many() {
        assert_eq!(
            many(token('a')).parse("aaabcd"),
            (Ok("aaa".to_string()), "bcd")
        );
        assert_eq!(
            many(token('a')).parse("aaabcd"),
            (Ok(vec!['a', 'a', 'a']), "bcd")
        );
        assert_eq!(many(token('b')).parse("abcd"), (Ok("".to_string()), "abcd"));
        assert_eq!(many(token('a')).parse("aaaa"), (Ok("aaaa".to_string()), ""));
        assert_eq!(
            many(many1(token('a'))).parse("aaabcd"),
            (Ok(vec!["aaa".to_string()]), "bcd")
        );
        assert_eq!(
            many(many1(token('b'))).parse("aaabcd"),
            (Ok(Vec::<String>::new()), "aaabcd")
        );
    }

    #[test]
    fn test_many1() {
        assert_eq!(
            many1(token('a')).parse("aaabcd"),
            (Ok("aaa".to_string()), "bcd")
        );
        assert_eq!(
            many1(token('a')).parse("abcd"),
            (Ok("a".to_string()), "bcd")
        );
        assert_eq!(
            many1(token('a')).parse("aaaa"),
            (Ok(vec!['a', 'a', 'a', 'a']), "")
        );
        assert_parse_err!(many1::<_, String>(token('b')).parse("abcd"), "abcd");
    }
}
