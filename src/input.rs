//! Types that represent parsable input.

use std::fmt::Debug;

use error::{Error, ParseResult};

pub struct Tokens<'a, I: Input>(Box<Iterator<Item = I::Item> + 'a>);

impl<'a, I: Input> Tokens<'a, I> {
    fn new<T: Iterator<Item = I::Item> + 'a>(t: T) -> Self {
        Tokens(Box::new(t))
    }
}

impl<'a, I: Input> Iterator for Tokens<'a, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub trait Input: Sized + Debug + Clone {
    type Item: Copy + PartialEq + Debug;

    fn peek(&self) -> Option<Self::Item>;
    fn pop(&mut self) -> Option<Self::Item>;
    fn tokens<I: Input<Item = Self::Item>>(&self) -> Tokens<I>;

    fn backup(&self) -> Self {
        self.clone()
    }
    fn restore(&mut self, backup: Self) {
        *self = backup;
    }

    fn ok<O>(self, result: O) -> ParseResult<Self, O> {
        (Ok(result), self)
    }
    fn err<O>(self, error: Error<Self>) -> ParseResult<Self, O> {
        (Err(error), self)
    }
}

pub trait Position<T>: Debug + Default + Clone {
    type Position: Clone + Ord;

    fn position(&self) -> Self::Position;
    fn update(&mut self, item: &T);
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct IndexPosition(usize);

impl Default for IndexPosition {
    fn default() -> Self {
        IndexPosition(0)
    }
}

impl<T> Position<T> for IndexPosition {
    type Position = usize;

    fn position(&self) -> Self::Position {
        self.0
    }

    fn update(&mut self, _: &T) {
        self.0 += 1;
    }
}

impl<X: Into<usize>> From<X> for IndexPosition {
    fn from(x: X) -> Self {
        IndexPosition(x.into())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct LinePosition {
    pub line: u32,
    pub column: u32,
}

impl Default for LinePosition {
    fn default() -> Self {
        LinePosition { line: 0, column: 0 }
    }
}

impl Position<char> for LinePosition {
    type Position = Self;

    fn position(&self) -> Self::Position {
        self.clone()
    }

    fn update(&mut self, item: &char) {
        if let &'\n' = item {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
}

impl From<(u32, u32)> for LinePosition {
    fn from((line, column): (u32, u32)) -> Self {
        LinePosition { line, column }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct State<I: Input, X: Position<I::Item>> {
    pub input: I,
    pub position: X,
}

impl<I: Input, X: Position<I::Item>> State<I, X> {
    pub fn new<Y: Into<X>>(input: I, position: Y) -> Self {
        State {
            input,
            position: position.into(),
        }
    }
}

impl<I: Input, X: Position<I::Item>> From<I> for State<I, X> {
    fn from(input: I) -> Self {
        State {
            input,
            position: Default::default(),
        }
    }
}

impl<I: Input, X: Position<I::Item>> Input for State<I, X> {
    type Item = I::Item;

    fn peek(&self) -> Option<Self::Item> {
        self.input.peek()
    }

    fn pop(&mut self) -> Option<Self::Item> {
        self.input.pop().map(|item| {
            self.position.update(&item);
            item
        })
    }

    fn tokens<T: Input<Item = Self::Item>>(&self) -> Tokens<T> {
        self.input.tokens()
    }
}

pub type SourceCode<I> = State<I, LinePosition>;

impl<'a> Input for &'a str {
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

    fn tokens<I: Input<Item = Self::Item>>(&self) -> Tokens<I> {
        Tokens::new(self.chars())
    }
}

impl Input for String {
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

    fn tokens<I: Input<Item = Self::Item>>(&self) -> Tokens<I> {
        Tokens::new(self.chars())
    }
}
