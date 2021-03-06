//! Traits and implementations representing positions in an `Stream` stream.

use std::fmt::{self, Debug, Display};
use std::hash::Hash;

use super::{RangeStream, Stream};

/// The Position trait defines types that keep track of the cursor position while parsing an
/// `Stream` stream.
pub trait Position<S: Stream>:
    Default + Debug + Display + Clone + Ord + PartialEq + Eq + Hash
{
    type Value: Ord;

    fn value(&self) -> Self::Value;
    fn update(&mut self, item: &S::Item);
    fn update_range(&mut self, range: &S::Range);

    fn fmt_msg(&self, msg: &str) -> String {
        format!("{} at {}", msg, self)
    }
}

/// NullPosition is a dummy `Position` for streams that don't keep track of their current position.
/// This is provided so that primitive types such as `&str` can implement `Stream`.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct NullPosition;

impl Display for NullPosition {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<S: Stream> Position<S> for NullPosition {
    type Value = ();

    fn value(&self) -> Self::Value {
        ()
    }

    fn update(&mut self, _: &S::Item) {}
    fn update_range(&mut self, _: &S::Range) {}

    fn fmt_msg(&self, msg: &str) -> String {
        String::from(msg)
    }
}

/// IndexPosition is a `Position` which is represented as an index.
/// This is useful for binary data or any kind of virtual input stream.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct IndexPosition(usize);

impl Default for IndexPosition {
    fn default() -> Self {
        IndexPosition(0)
    }
}

impl Display for IndexPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "index {}", self.0)
    }
}

impl<S: Stream> Position<S> for IndexPosition {
    type Value = usize;

    fn value(&self) -> Self::Value {
        self.0
    }

    fn update(&mut self, _: &S::Item) {
        self.0 += 1;
    }

    fn update_range(&mut self, range: &S::Range) {
        self.0 += range.len();
    }
}

impl From<usize> for IndexPosition {
    fn from(x: usize) -> Self {
        IndexPosition(x)
    }
}

impl From<LinePosition> for IndexPosition {
    fn from(LinePosition { line, column }: LinePosition) -> Self {
        IndexPosition((line * column) as usize)
    }
}

/// LinePosition is a `Position` which is represented as a line number and column number.
/// This is primarily useful for parsing text files or anything that has multiple lines.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct LinePosition {
    pub line: u32,
    pub column: u32,
}

impl Default for LinePosition {
    fn default() -> Self {
        LinePosition { line: 1, column: 1 }
    }
}

impl Display for LinePosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {}, col {}", self.line, self.column)
    }
}

impl<'a> Position<&'a str> for LinePosition {
    type Value = (u32, u32);

    fn value(&self) -> Self::Value {
        (self.line, self.column)
    }

    fn update(&mut self, item: &char) {
        if let &'\n' = item {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }

    fn update_range(&mut self, range: &&str) {
        for item in range.chars() {
            self.update(&item);
        }
    }
}

impl From<(u32, u32)> for LinePosition {
    fn from((line, column): (u32, u32)) -> Self {
        LinePosition { line, column }
    }
}
