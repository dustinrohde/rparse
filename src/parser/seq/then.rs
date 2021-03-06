use std::marker::PhantomData;

use {ParseResult, Parser, Stream};

pub struct Then<I, L, R> {
    p1: L,
    p2: R,
    _marker: PhantomData<I>,
}

impl<I, L, R> Parser for Then<I, L, R>
where
    L: Parser,
    R: Parser<Stream = L::Stream, Output = L::Output>,
    I: std::iter::FromIterator<L::Output>,
{
    type Stream = L::Stream;
    type Output = I;

    fn parse_partial(&mut self, stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        let (first, stream) = self.p1.parse_partial(stream)?;
        let (second, stream) = self.p2.parse_partial(stream)?;
        stream.ok(first.into_iter().chain(second.into_iter()).collect())
    }
}

pub fn then<I, L, R>(p1: L, p2: R) -> Then<I, L, R>
where
    L: Parser,
    R: Parser<Stream = L::Stream, Output = L::Output>,
    I: std::iter::FromIterator<L::Output>,
{
    Then {
        p1,
        p2,
        _marker: PhantomData,
    }
}

#[cfg(test)]
mod test {
    use parser::item::item;
    use stream::IndexedStream;
    use {Error, Parser};

    #[test]
    fn test_then() {
        let mut parser = item(b'X').then(item(b'O'));
        test_parser!(IndexedStream<&str> => Vec<char> | parser, {
            "XO" => ok("XO".chars().collect(), ("", 2)),
            "XOXO" => ok("XO".chars().collect(), ("XO", 2)),
            "XY" => err(Error::item('Y').expected_item('O').at(1)),
            "ZY" => err(Error::item('Z').expected_item('X').at(0)),
        });
    }
}
