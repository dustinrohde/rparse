//! A collection of various parsers and combinators.
//!
//! Defines the `Parser` trait.

#[cfg(test)]
#[macro_use]
mod test_utils;

#[macro_use]
pub mod choice;
pub mod combinator;
pub mod token;
pub mod transform;

use self::choice::{and, or, And, Or};
use self::transform::{map, then, Map, Then};
use error::ParseResult;
use input::Input;

pub trait Parser {
    type Input: Input;
    type Output;

    fn parse_input(&mut self, Self::Input) -> ParseResult<Self::Input, Self::Output>;

    fn parse(&mut self, input: Self::Input) -> ParseResult<Self::Input, Self::Output>
    where
        Self: Sized,
    {
        let backup = input.backup();
        let mut result = self.parse_input(input);
        if let (Err(_), ref mut input) = result {
            input.restore(backup);
        }
        result
    }

    fn map<F, O>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> O,
    {
        map(self, f)
    }

    fn then<F, O>(self, f: F) -> Then<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output, Self::Input) -> O,
    {
        then(self, f)
    }

    fn and<P>(self, other: P) -> And<Self, P>
    where
        Self: Sized,
        P: Parser<Input = Self::Input, Output = Self::Output>,
    {
        and(self, other)
    }

    fn or<P>(self, other: P) -> Or<Self, P>
    where
        Self: Sized,
        P: Parser<Input = Self::Input, Output = Self::Output>,
    {
        or(self, other)
    }
}

impl<'a, I: Input, O> Parser for FnMut(I) -> ParseResult<I, O> + 'a {
    type Input = I;
    type Output = O;

    fn parse_input(&mut self, input: Self::Input) -> ParseResult<Self::Input, Self::Output> {
        self(input)
    }
}

impl<I: Input, O> Parser for fn(I) -> ParseResult<I, O> {
    type Input = I;
    type Output = O;

    fn parse_input(&mut self, input: Self::Input) -> ParseResult<Self::Input, Self::Output> {
        self(input)
    }
}

pub fn parser<I: Input, O>(f: fn(I) -> ParseResult<I, O>) -> fn(I) -> ParseResult<I, O> {
    f
}
