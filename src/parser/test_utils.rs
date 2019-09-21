#[cfg(test)]
use {ParseResult, Stream};

#[cfg(test)]
pub fn ok_result<S, O>(output: O, stream: S) -> ParseResult<S, O>
where
    S: Stream,
{
    ParseResult::new(Ok(Some(output)), stream)
}

#[macro_export]
macro_rules! test_parser {
    // New API

    ($stream_ty:ty => $output_ty:ty | $p:expr, {
        $($into_input:expr => $assertion:ident $expected:expr),+ $(,)*
    }) => {
        $(
            test_parser!(@dispatch $assertion $stream_ty, $output_ty, $p, $into_input, $expected);
        )+
    };

    (@dispatch ok $stream_ty:ty, $output_ty:ty, $p:expr, $into_input:expr, $expected:expr) => {
        let input: $stream_ty = $into_input.into();
        let result: $crate::ParseResult<$stream_ty, $output_ty> = $p.parse(input.clone());
        let (expected_result, into_expected_stream): ($output_ty, _) = $expected;
        let expected_stream: $stream_ty = into_expected_stream.into();
        assert_eq!(result.data.unwrap().unwrap(), expected_result);
        assert_eq!(result.stream, expected_stream);
    };

    (@dispatch noop $stream_ty:ty, $output_ty:ty, $p:expr, $into_input:expr, $expected:expr) => {
        let input: $stream_ty = $into_input.into();
        let result: $crate::ParseResult<$stream_ty, $output_ty> = $p.parse(input.clone());
        assert_eq!(result.data.unwrap(), None);
        assert_eq!(result.stream, input);
    };

    (@dispatch err $stream_ty:ty, $output_ty:ty, $p:expr, $into_input:expr, $expected:expr) => {
        let input: $stream_ty = $into_input.into();
        let result: $crate::ParseResult<$stream_ty, $output_ty> = $p.parse(input.clone());
        let data = result.data.expect_err("assertion failed: expected an Err(_)");
        let expected_result: $crate::error::Errors<$stream_ty> = $expected.into();
        let expected_stream = input;
        assert_eq!(data, expected_result);
        assert_eq!(result.stream, expected_stream);
    };
}
