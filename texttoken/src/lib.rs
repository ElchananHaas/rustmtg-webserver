use std::{borrow::Cow, ops::RangeFrom};

use nom::{Compare, CompareResult, FindToken};
pub type Token = Cow<'static, str>;
#[macro_export]
macro_rules! tokens{
    ($($e:expr),*) => {
        Tokens::from_array(
            &[
                $(
                    $e.into(),
                )*
            ]
        )
    };
}
#[macro_export]
macro_rules! owned_tokens{
    ($($e:expr),*) => {
        vec![
            $(
                Cow::<'static, str>::Borrowed($e),
            )*
        ]
    };
}
#[derive(Debug)]
#[repr(transparent)]
pub struct Tokens {
    pub tokens: [Token],
}
impl Tokens {
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
    pub fn from_array<'a>(tokens: &'a [Token]) -> &'a Self {
        unsafe { std::mem::transmute(tokens) }
    }
    pub fn empty() -> &'static Self {
        Self::from_array(&[])
    }
}
impl<'a> FindToken<Token> for &'a Tokens {
    fn find_token(&self, token: Token) -> bool {
        for x in &self.tokens {
            if *x == token {
                return true;
            }
        }
        false
    }
}
impl<'a> nom::InputLength for &'a Tokens {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}
impl<'a> nom::InputTake for &'a Tokens {
    fn take(&self, x: usize) -> Self {
        Tokens::from_array(&self.tokens[..x])
    }
    fn take_split(&self, x: usize) -> (Self, Self) {
        (
            Tokens::from_array(&self.tokens[x..]),
            Tokens::from_array(&self.tokens[..x]),
        )
    }
}

impl<'a> nom::UnspecializedInput for &'a Tokens {}

impl<'a, 'b> Compare<&'b Tokens> for &'a Tokens {
    fn compare(&self, t: &'b Tokens) -> CompareResult {
        let pos = self
            .tokens
            .iter()
            .zip(t.tokens.iter())
            .position(|(a, b)| a != b);
        match pos {
            Some(_) => CompareResult::Error,
            None => {
                if self.len() >= t.len() {
                    CompareResult::Ok
                } else {
                    CompareResult::Incomplete
                }
            }
        }
    }
    fn compare_no_case(&self, _: &'b Tokens) -> CompareResult {
        panic!("Tokens don't support case insensitice comparison")
    }
}

fn mul_loc(x: (usize, Token)) -> (usize, Token) {
    (x.0, x.1)
}
impl<'a> nom::InputIter for &'a Tokens {
    type Item = Token;
    type Iter = std::iter::Map<
        std::iter::Enumerate<std::iter::Cloned<std::slice::Iter<'a, Token>>>,
        fn((usize, Token)) -> (usize, Token),
    >;
    type IterElem = std::iter::Cloned<std::slice::Iter<'a, Token>>;
    fn iter_indices(&self) -> Self::Iter {
        self.tokens.iter().cloned().enumerate().map(mul_loc)
    }
    fn iter_elements(&self) -> Self::IterElem {
        self.tokens.iter().cloned()
    }
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        for (o, c) in self.iter_indices() {
            if predicate(c) {
                return Some(o);
            }
        }
        None
    }
    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        if self.len() >= count {
            Ok(count)
        } else {
            Err(nom::Needed::new(count - self.len()))
        }
    }
}
impl std::ops::Index<usize> for Tokens {
    type Output = Token;
    fn index(&self, x: usize) -> &Token {
        &self.tokens[x]
    }
}

impl std::ops::Index<RangeFrom<usize>> for Tokens {
    type Output = Tokens;
    fn index(&self, x: RangeFrom<usize>) -> &Tokens {
        Tokens::from_array(&self.tokens[x])
    }
}
