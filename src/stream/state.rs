use super::position::Position;
use super::{RangeStream, Stream, ToStream, Tokens};
// use error::{Error, Errors};

#[derive(Debug, Clone, PartialEq)]
pub struct State<S: RangeStream, X: Position<S::Item>> {
    pub stream: S,
    pub position: X,
}

impl<S: RangeStream, X: Position<S::Item>> State<S, X> {
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

impl<S: RangeStream, X: Position<S::Item>> From<S> for State<S, X> {
    fn from(stream: S) -> Self {
        State {
            stream,
            position: Default::default(),
        }
    }
}

impl<S: RangeStream, X: Position<S::Item>, T: Into<X>> From<(S, T)> for State<S, X> {
    fn from((stream, pos): (S, T)) -> Self {
        State {
            stream,
            position: pos.into(),
        }
    }
}

impl<S, X> Stream for State<S, X>
where
    S: RangeStream + ToStream<S>,
    S::Item: ToStream<Self>,
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

impl<S, X> ToStream<State<S, X>> for S::Item
where
    S: RangeStream + ToStream<S>,
    S::Item: ToStream<S>,
    X: Position<S::Item>,
{
    fn to_stream(self) -> State<S, X> {
        let s = self.to_stream();
        State::from(s)
    }
}

impl<X> ToStream<State<String, X>> for String
where
    X: Position<<String as Stream>::Item>,
{
    fn to_stream(self) -> State<String, X> {
        self.into()
    }
}

impl<'a, X> ToStream<State<&'a str, X>> for &'a str
where
    X: Position<<&'a str as Stream>::Item>,
{
    fn to_stream(self) -> State<&'a str, X> {
        self.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stream::{FromStream, NullPosition};

    #[test]
    fn test_to_stream_string_state_for_string() {
        assert_eq!(
            ToStream::<State<String, NullPosition>>::to_stream(String::from("yee")),
            State::from("yee".to_string())
        );
        assert_eq!(
            <State<String, NullPosition> as FromStream<String>>::from_stream("yee".to_string()),
            State::from("yee".to_string())
        );
    }

    #[test]
    fn test_to_stream_str_state_for_str() {
        assert_eq!(
            ToStream::<State<&str, NullPosition>>::to_stream("yee"),
            State::from("yee")
        );
        assert_eq!(
            <State<&str, NullPosition> as FromStream<&str>>::from_stream("yee"),
            State::from("yee")
        );
    }

    #[test]
    fn test_to_stream_string_state_for_char() {
        assert_eq!(
            ToStream::<State<String, NullPosition>>::to_stream('x'),
            State::from("x".to_string())
        );
    }

    #[test]
    fn test_to_stream_str_state_for_char() {
        assert_eq!(
            ToStream::<State<&str, NullPosition>>::to_stream('x'),
            State::from("x")
        );
    }
}
