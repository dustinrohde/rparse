use super::position::Position;
use super::{RangeStream, Stream, ToStream, Tokens};
// use error::{Error, Errors};

#[derive(Debug, Clone, PartialEq)]
pub struct State<S: Stream, X: Position<S::Item>> {
    pub stream: S,
    pub position: X,
}

impl<S: Stream, X: Position<S::Item>> State<S, X> {
    pub fn new<T: Into<X>>(stream: S, position: T) -> Self {
        State {
            stream,
            position: position.into(),
        }
    }

    //     pub fn add_error(&self, error: Error<S>) -> Errors<Self, X> {
    //         Errors::new(self.position.clone(), error)
    //     }
}

impl<S: Stream, X: Position<S::Item>> From<S> for State<S, X> {
    fn from(stream: S) -> Self {
        State {
            stream,
            position: Default::default(),
        }
    }
}

impl<S: Stream, X: Position<S::Item>, T: Into<X>> From<(S, T)> for State<S, X> {
    fn from((stream, pos): (S, T)) -> Self {
        State {
            stream,
            position: pos.into(),
        }
    }
}

impl<S, X> Stream for State<S, X>
where
    S: RangeStream,
    X: Position<S::Item>,
{
    type Item = S::Item;
    type Range = S::Range;
    type Position = X;

    fn from_token(token: Self::Item) -> Self {
        Self::Range::from_token(token).into()
    }

    fn from_range(range: Self::Range) -> Self {
        Self::Range::from_range(range).into()
    }

    fn peek(&self) -> Option<Self::Item> {
        self.stream.peek()
    }

    fn pop(&mut self) -> Option<Self::Item> {
        self.stream.pop().map(|item| {
            self.position.update(&item);
            item
        })
    }

    fn tokens(&self) -> Tokens<Self::Item> {
        self.stream.tokens()
    }

    fn range(&mut self, idx: usize) -> Option<Self::Range> {
        self.stream.range(idx).map(|range| {
            for token in range.tokens() {
                self.position.update(&token);
            }
            range
        })
    }

    fn position(&self) -> Self::Position {
        self.position.clone()
    }
}

impl<'a, S, X> ToStream<State<S, X>> for S
where
    S: RangeStream,
    X: Position<S::Item>,
{
    fn to_stream(self) -> State<S, X> {
        self.into()
    }
}

impl<'a, X> ToStream<State<&'a str, X>> for char
where
    X: Position<char>,
{
    fn to_stream(self) -> State<&'a str, X> {
        let s: &'a str = self.to_stream();
        s.into()
    }
}

impl<'a, X> ToStream<State<String, X>> for char
where
    X: Position<char>,
{
    fn to_stream(self) -> State<String, X> {
        let s: String = self.to_stream();
        s.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stream::{FromStream, NullPosition};

    #[test]
    fn test_to_stream_state_string_for_string() {
        assert_eq!(
            ToStream::<State<String, NullPosition>>::to_stream(String::from("yee")),
            State::from("yee".to_string())
        );
        assert_eq!(
            <State<String, NullPosition> as FromStream<String>>::from_stream("yee".to_string()),
            State::from("yee".to_string())
        );
    }

    // #[test]
    // fn test_to_stream_state_str_for_str() {
    //     assert_eq!(
    //         <String as ToStream<String>>::to_stream("yee".to_string()),
    //         String::from("yee")
    //     );
    //     assert_eq!(String::from_stream("yee".to_string()), String::from("yee"));
    // }

    // #[test]
    // fn test_to_stream_state_str_for_char() {
    //     assert_eq!('x'.to_stream(), State::from("x"));
    // assert_eq!(<char as ToStream<State<&str, Position<char>>>::to_stream('x'), State::new("x"));
    // }

    // #[test]
    // fn test_to_stream_string_for_char() {
    //     assert_eq!(
    //         <char as ToStream<String>>::to_stream('x'),
    //         String::from("x")
    //     );
    // }

    // #[test]
    // fn test_to_stream_string_for_str() {
    //     assert_eq!(
    //         <&str as ToStream<String>>::to_stream("yee"),
    //         String::from("yee")
    //     );
    //     assert_eq!(String::from_stream("yee"), String::from("yee"));
    // }
}
