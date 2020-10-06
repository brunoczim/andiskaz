use crate::string::{TermGrapheme, TermString};
use std::{
    fmt,
    ops::{Range, RangeFrom, RangeFull, RangeTo},
};

#[inline(never)]
#[cold]
fn index_panic<I>(count: usize, index: I) -> !
where
    I: fmt::Debug,
{
    panic!(
        "TermString index out of bounds: grapheme ([`TermGrapheme`]) count is \
         {} but index {:?}",
        count, index
    )
}

/// Specifies usable indices for a `TermString`.
pub trait Index {
    /// Output of the indexing operation.
    type Output;

    /// Tries to index the `TermString` and returns `None` if out of bounds.
    fn get(self, tstring: &TermString) -> Option<Self::Output>;
    /// Indexes the `TermString` or panics if out of bounds.
    ///
    /// # Panics
    /// Panics if out of bounds.
    fn index(self, tstring: &TermString) -> Self::Output;
}

impl Index for usize {
    type Output = TermGrapheme;

    fn get(self, tstring: &TermString) -> Option<Self::Output> {
        tstring.into_iter().nth(self)
    }

    fn index(self, tstring: &TermString) -> Self::Output {
        self.get(tstring)
            .unwrap_or_else(|| index_panic(tstring.count_graphemes(), self))
    }
}

impl Index for Range<usize> {
    type Output = TermString;

    fn get(self, tstring: &TermString) -> Option<Self::Output> {
        let mut iter = tstring.indices();
        for _ in 0 .. self.start {
            iter.next()?;
        }
        let (start, _) = iter.next()?;
        for _ in self.start + 1 .. self.end {
            iter.next()?;
        }
        let end = iter.next().map_or(tstring.len(), |(index, _)| index);
        let range =
            start + tstring.range.start .. end + tstring.range.start;
        Some(TermString { alloc: tstring.alloc.clone(), range })
    }

    fn index(self, tstring: &TermString) -> Self::Output {
        self.clone()
            .get(tstring)
            .unwrap_or_else(|| index_panic(tstring.count_graphemes(), self))
    }
}

impl Index for RangeTo<usize> {
    type Output = TermString;

    fn get(self, tstring: &TermString) -> Option<Self::Output> {
        (0 .. self.end).get(tstring)
    }

    fn index(self, tstring: &TermString) -> Self::Output {
        self.clone()
            .get(tstring)
            .unwrap_or_else(|| index_panic(tstring.count_graphemes(), self))
    }
}

impl Index for RangeFrom<usize> {
    type Output = TermString;

    fn get(self, tstring: &TermString) -> Option<Self::Output> {
        let mut iter = tstring.indices();
        for _ in 0 .. self.start {
            iter.next()?;
        }
        let start =
            iter.next().map_or(tstring.alloc.len(), |(index, _)| index);
        let end = tstring.alloc.len();
        let range =
            start + tstring.range.start .. end + tstring.range.start;
        Some(TermString { alloc: tstring.alloc.clone(), range })
    }

    fn index(self, tstring: &TermString) -> Self::Output {
        self.clone()
            .get(tstring)
            .unwrap_or_else(|| index_panic(tstring.count_graphemes(), self))
    }
}

impl Index for RangeFull {
    type Output = TermString;

    fn get(self, tstring: &TermString) -> Option<Self::Output> {
        Some(tstring.clone())
    }

    fn index(self, tstring: &TermString) -> Self::Output {
        tstring.clone()
    }
}
