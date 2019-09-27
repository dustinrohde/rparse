//! Parsers that parse a continuous series of tokens.

use error::{Error, ParseResult};
use parser::Parser;
use stream::{Position, RangeStream, Stream};

pub struct Range<S: Stream> {
    range: S::Range,
}

impl<S: Stream> Parser for Range<S> {
    type Stream = S;
    type Output = S::Range;

    fn parse_lazy(&mut self, mut stream: Self::Stream) -> ParseResult<Self::Stream, Self::Output> {
        let idx = self.range.len();
        let mut start_pos = stream.position().clone();
        let mut result = match stream.range(idx) {
            Some(mut range) => {
                if range == self.range {
                    stream.ok(range)
                } else {
                    let located = range
                        .tokens()
                        .zip(self.range.tokens())
                        .enumerate()
                        .find(|&(_, (left, right))| left != right);
                    match located {
                        Some((i, (left, _))) => {
                            let range = range.range(i).unwrap();
                            start_pos.update_range(&range);
                            stream.err(Error::unexpected_token(left))
                        }
                        None => stream.err(Error::eoi()),
                    }
                }
            }
            None => stream.err(Error::eoi()),
        };
        if let Err((ref mut errors, _)) = result {
            errors.position = start_pos;
        }
        result
    }

    fn expected_errors(&self) -> Vec<Error<Self::Stream>> {
        vec![Error::expected_range(self.range.clone())]
    }
}

pub fn range<S: Stream>(range: &'static str) -> Range<S> {
    Range {
        range: S::Range::from_str(range),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stream::IndexedStream;
    use Error::*;

    #[test]
    fn test_range() {
        let mut parser = range("def");
        test_parser!(IndexedStream<&str> => &str | parser, {
            "def" => ok("def", ("", 3)),
            "defcon" => ok("def", ("con", 3)),
            "" => err(0, vec![Error::eoi(), Error::expected_range("def")]),
            "de" => err(0, vec![Error::eoi(), Error::expected_range("def")]),
            "dr" => err(0, vec![Error::eoi(), Error::expected_range("def")]),
            "deg" => err(2, vec![Unexpected('g'.into()), Error::expected_range("def")]),
        });
    }
}

// TODO
// pub struct SatisfyRange
