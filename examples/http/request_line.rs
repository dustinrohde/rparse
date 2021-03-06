use std::str::FromStr;

use rparse::parser::{
    item::{ascii, item, satisfy},
    parser,
    range::range,
    repeat::{many, many1},
};
use rparse::stream::StreamItem;
use rparse::{Parser, Stream};

use common::crlf;

enum HTTPVersion {
    V1,
    V11,
    V2,
}

impl FromStr for HTTPVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Self::V1),
            "1.1" => Ok(Self::V11),
            "2" => Ok(Self::V2),
            _ => Err(format!("invalid http version '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestLine {
    pub method: String,
    pub uri: String,
    pub version: String,
}

pub fn request_line<S>() -> impl Parser<Stream = S, Output = RequestLine>
where
    S: Stream,
{
    many::<(), _>(crlf()).with(parser(|s: S| {
        let (method, s) = http_method().must_parse(s)?;
        let (uri, s) = item(b' ').with(uri()).must_parse(s)?;
        let (version, s) = item(b' ').with(http_version()).must_parse(s)?;
        s.ok(RequestLine {
            method,
            uri,
            version,
        })
    }))
}

fn http_version<S>() -> impl Parser<Stream = S, Output = String>
where
    S: Stream,
{
    // an HTTP version is the text "HTTP/"
    range("HTTP/")
        // followed by a version number
        .with(choice![range("1.1"), range("1"), range("2")])
        .as_string()
}

fn http_method<S>() -> impl Parser<Stream = S, Output = String>
where
    S: Stream,
{
    choice![
        range("GET"),
        range("PUT"),
        range("POST"),
        range("HEAD"),
        range("PATCH"),
        range("TRACE"),
        range("DELETE"),
        range("OPTIONS"),
        range("CONNECT"),
    ]
    .as_string()
}

fn uri<S>() -> impl Parser<Stream = S, Output = String>
where
    S: Stream,
{
    // a URI is
    seq![
        // a scheme
        uri_scheme(),
        // optionally followed by a path
        uri_path().optional(),
    ]
    .collect()
}

fn uri_scheme<S>() -> impl Parser<Stream = S, Output = String>
where
    S: Stream,
{
    // a URI scheme is
    (
        // a scheme identifier
        many1::<Vec<_>, _>(ascii::letter()).collect_string(),
        // followed by a delimiter
        range("://").as_string(),
    )
        .map(|(r0, r1): (String, String)| format!("{}{}", r0, r1))
}

fn uri_path<S>() -> impl Parser<Stream = S, Output = String>
where
    S: Stream,
{
    // a URI path is either
    choice![
        concat![
            // a slash
            item(b'/').wrap(),
            // followed by zero or more URI segments separated by slashes
            concat![
                uri_segment(),
                many::<Vec<_>, _>(item(b'/').wrap().or(uri_segment())).flatten(),
            ]
            .optional()
        ],
        // or a URI segment followed by zero or more URI segments separated by slashes
        concat![
            uri_segment(),
            many::<Vec<_>, _>(item(b'/').wrap().or(uri_segment())).flatten(),
        ],
    ]
    .collect_string()
}

fn uri_segment<S>() -> impl Parser<Stream = S, Output = Vec<S::Item>>
where
    S: Stream,
{
    // a URI segment is one or more
    many1::<Vec<_>, _>(choice![
        // percent-encoded octets
        percent_encoded(),
        // and URI-safe character sequences
        many1(uri_token()),
    ])
    .flatten()
}

fn uri_token<S>() -> impl Parser<Stream = S, Output = S::Item>
where
    S: Stream,
{
    satisfy(|item: &S::Item| {
        item.is_ascii_alphanumeric()
            || [b'-', b'_', b'.', b'!', b'~', b'*', b'\'', b'(', b')']
                .iter()
                .any(|&b| &S::Item::from(b) == item)
    })
}

fn percent_encoded<S>() -> impl Parser<Stream = S, Output = Vec<S::Item>>
where
    S: Stream,
{
    item(b'%').then(ascii::hexdigit()).append(ascii::hexdigit())
}

#[cfg(test)]
mod test {
    use super::*;
    use rparse::error::{Error, Info};
    use rparse::stream::IndexedStream;

    // TODO: [u8]
    #[test]
    fn test_http_method() {
        let into_expected: Vec<_> = [
            "GET", "PUT", "POST", "HEAD", "PATCH", "TRACE", "DELETE", "OPTIONS", "CONNECT",
        ]
        .iter()
        .map(|s| Info::Range(s.as_bytes()))
        .collect();

        test_parser!(IndexedStream<&[u8]> => String | http_method(), {
            &b"GET"[..] => ok("GET".into(), (&b""[..], 3)),
            &b"HEAD\n/"[..] => ok("HEAD".into(), (&b"\n/"[..], 4)),
            &b"GET http://"[..] => ok("GET".into(), (&b" http://"[..], 3)),
        });

        test_parser!(IndexedStream<&[u8]> => String | http_method(), {
            &b"PUPPYDOG"[..] => err(Error::item(b'P').expected_one_of(into_expected).at(0)),
        });

        assert_eq!(
            http_method().parse("TRACE it"),
            Ok((Some("TRACE".into()), " it"))
        );
    }

    #[test]
    fn test_percent_encoded() {
        test_parser!(&str => String | percent_encoded().collect::<String>(), {
            "%A9" => ok("%A9".into(), ""),
            "%0f/hello" => ok("%0f".into(), "/hello"),
            "" => err(Error::eoi().expected_item('%')),
            "%%0f" => err(Error::item('%').expected("a hexadecimal digit")),
            "%xy" => err(Error::item('x').expected("a hexadecimal digit")),
        });
    }

    #[test]
    fn test_uri_path() {
        test_parser!(IndexedStream<&str> => String | uri_path(), {
            "/" => ok("/".into(), ("", 1)),
            "foo" => ok("foo".into(), ("", 3)),
            "/my_img.jpeg" => ok("/my_img.jpeg".into(), ("", 12)),
            "foo/x%20y/z.gif/" => ok("foo/x%20y/z.gif/".into(), ("", 16)),
            "/%%bc" => ok("/".into(), ("%%bc", 1)),
            "//a/" => ok("/".into(), ("/a/", 1)),
        });
    }
}
