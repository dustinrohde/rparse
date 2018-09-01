use super::position::{NullPosition, Position};
use super::{Stream, Tokens};
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

impl<S: Stream<Position = NullPosition>, X: Position<S::Item>> Stream for State<S, X> {
    type Item = S::Item;
    type Range = S::Range;
    type Position = X;

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
