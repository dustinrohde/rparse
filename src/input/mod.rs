//! Traits and implementations defining parsable input streams.

pub mod position;
pub mod state;

use std::fmt::Debug;

pub use self::position::Position;
use self::position::{IndexPosition, LinePosition};
use self::state::State;
use error::{Error, ParseResult};

/// SourceCode is a type alias for str `Stream` positioned by rows and columns.
pub type SourceCode = State<&'static str, LinePosition>;

/// IndexedStream is a type alias for `Stream` positioned by its index.
pub type IndexedStream<I> = State<I, IndexPosition>;

/// Tokens is an iterator over the tokens of some `Stream`.
/// It is returned by the `tokens` method of `Stream`.
pub struct Tokens<'a, T>(Box<Iterator<Item = T> + 'a>);

impl<'a, T> Tokens<'a, T> {
    fn new<I: Iterator<Item = T> + 'a>(iter: I) -> Self {
        Tokens(Box::new(iter))
    }
}

impl<'a, T> Iterator for Tokens<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// The Stream trait represents data that can be consumed by a `Parser`.
pub trait Stream: Sized + Debug + Clone {
    /// The type of a single token.
    type Item: Copy + PartialEq + Debug;

    /// Returns the next token in the stream without consuming it.
    /// If there are no more tokens, returns `None`.
    fn peek(&self) -> Option<Self::Item>;

    /// Removes and returns the next token in the stream.
    fn pop(&mut self) -> Option<Self::Item>;

    /// Returns an iterator over remaining tokens in the stream.
    /// Does not consume any input.
    fn tokens(&self) -> Tokens<Self::Item>;

    /// Return a snapshot of the current stream.
    /// The returned snapshot can be restored with `restore()`.
    fn backup(&self) -> Self {
        self.clone()
    }
    /// Reset the stream to the given state.
    /// This method is intended for use with the `backup()` method.
    fn restore(&mut self, backup: Self) {
        *self = backup;
    }

    /// Return the given parse output as a `ParseResult`, using `Self` as the `Stream` type.
    fn ok<O>(self, result: O) -> ParseResult<Self, O> {
        (Ok(result), self)
    }
    /// Return the given parse error as a `ParseResult`, using `Self` as the `Stream` type.
    fn err<O>(self, error: Error<Self>) -> ParseResult<Self, O> {
        (Err(error), self)
    }
}

impl<'a> Stream for &'a str {
    type Item = char;

    fn peek(&self) -> Option<Self::Item> {
        self.chars().next()
    }

    fn pop(&mut self) -> Option<Self::Item> {
        let mut iter = self.char_indices();
        iter.next().map(|(_, c)| {
            match iter.next() {
                Some((n, _)) => *self = &self[n..],
                None => *self = &self[..0],
            }

            c
        })
    }

    fn tokens(&self) -> Tokens<Self::Item> {
        Tokens::new(self.chars())
    }
}

impl Stream for String {
    type Item = char;

    fn peek(&self) -> Option<Self::Item> {
        self.chars().next()
    }

    fn pop(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            Some(self.remove(0))
        }
    }

    fn tokens(&self) -> Tokens<Self::Item> {
        Tokens::new(self.chars())
    }
}
