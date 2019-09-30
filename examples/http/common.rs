use rparse::parser::{
    item::{one_of, satisfy},
    range::range,
    repeat::many1,
};
use rparse::stream::StreamItem;
use rparse::{Parser, Stream};

pub static SEPARATORS: &'static [u8] = &[
    b'(', b')', b'<', b'>', b'@', b',', b';', b':', b'\\', b'"', b'/', b'[', b']', b'?', b'=',
    b'{', b'}', b' ', b'\t',
];

/// Parses a token.
///
/// Tokens are commonly used to define values between separators. In most cases, non-tokens can
/// only be used if escaped (within quotations (" ") or by a backslash (\)).
pub fn token<S: Stream>() -> impl Parser<Stream = S, Output = S::Item> {
    satisfy(|item: &S::Item| {
        let separators = SEPARATORS
            .iter()
            .map(|&b| b.into())
            .collect::<Vec<S::Item>>();
        !(item.is_ascii_control() || separators.contains(item))
    })
    .expect("a token")
}

pub fn lws<S: Stream>() -> impl Parser<Stream = S, Output = ()> {
    many1(one_of(&[b' ', b'\t']).map(|_| ()))
}

fn crlf<S: Stream>() -> impl Parser<Stream = S> {
    range("\r\n")
}

#[cfg(test)]
mod test {
    use super::*;
    use rparse::stream::IndexedStream;
    use rparse::Error;

    #[test]
    fn test_token() {
        let mut parser = token();
        test_parser!(IndexedStream<&str> => char | parser, {
            "a" => ok('a', ("", 1)),
            "11" => ok('1', ("1", 1)),
            "_ab" => ok('_', ("ab", 1)),
            "" => err(0, vec![Error::eoi(), Error::expected("a token")]),
        });

        for c in 0u8..=32 {
            let input = format!("{}_foo", c as char);
            let stream = IndexedStream::<&str>::from(input.as_ref());
            assert_eq!(
                token().parse(stream.clone()),
                Err((
                    (
                        0,
                        vec![
                            Error::unexpected_item(c as char),
                            Error::expected("a token")
                        ]
                    )
                        .into(),
                    stream
                )),
                "unexpectedly parsed '{}': should fail parsing control characters",
                c as char,
            );
        }
        for &item in SEPARATORS.iter() {
            let input = [item, b'\n'];
            let stream = IndexedStream::from(&input[..]);
            assert_eq!(
                token().parse(stream.clone()),
                Err((
                    (
                        0,
                        vec![Error::unexpected_item(item), Error::expected("a token")]
                    )
                        .into(),
                    stream
                )),
                "unexpectedly parsed '{}': should fail parsing SEPARATORS",
                item as char,
            );
        }
    }

    #[test]
    fn test_lws() {
        let mut parser = lws();
        test_parser!(IndexedStream<&str> => () | parser, {
            " " => ok((), ("", 1)),
            "\t" => ok((), ("", 1)),
            "  \t\t " => ok((), ("", 5)),
            "\t \tfoo" => ok((), ("foo", 3)),
            " foo" => ok((), ("foo", 1)),
            " \t foo" => ok((), ("foo", 3)),
            "" => err(0, vec![Error::eoi(), Error::expected_one_of(vec![b' ', b'\t'])]),
            "\r\n" => err(0, vec![
                Error::unexpected_item('\r'),
                Error::expected_one_of(vec![b' ', b'\t']),
            ]),
        });
    }
}
