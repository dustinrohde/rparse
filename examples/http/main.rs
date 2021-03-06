#![feature(trait_alias)]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde_json;

use serde_json::Value;

#[macro_use]
extern crate rparse;

mod common;
mod headers;
mod json;
mod request_line;

use rparse::parser::{
    item::{ascii, eoi_},
    repeat::many,
};
use rparse::stream::SourceCode;
use rparse::{Parser, Stream};

use common::crlf;
use headers::{headers, Headers};
use json::json_value;
use request_line::{request_line, RequestLine};

static INPUT: &'static str = "\
GET https://foo.bar/I%20like%20/50 HTTP/1.1\r
Accept: *\r
Content-Type: application/json\r
\r
{
    \"foo\": true,
    \"bar\": 3
}\r
";

fn request<S: Stream>() -> impl Parser<Stream = S, Output = (RequestLine, Headers, Option<Value>)> {
    (
        request_line().skip(crlf()),
        headers(),
        choice![
            crlf().with(json_value().skip(crlf())).map(|v| Some(v)),
            many::<(), _>(crlf()).map(|_| None),
        ],
    )
        .skip(many::<Vec<_>, _>(ascii::whitespace()))
        .skip(eoi_())
}

fn main() {
    let stream = SourceCode::from(INPUT);
    match request().must_parse(stream) {
        Ok((result, _)) => {
            println!("Parsing succeeded!");
            dbg!(result);
        }
        Err((err, _)) => {
            println!("Parsing failed!");
            dbg!(err);
        }
    };
}
