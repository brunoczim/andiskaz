use crate::string::{TermGrapheme, TermString};
use std::fmt;
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

/// Iterator over the `TermString`'s grapheme cluster ([`TermGrapheme`])s
/// indices and over the grapheme cluster ([`TermGrapheme`])s themselves.
pub struct TermStringIndices<'term_string> {
    base: TermString,
    prev_index: usize,
    next_index: usize,
    indices: GraphemeIndices<'term_string>,
}

impl<'term_string> TermStringIndices<'term_string> {
    pub(crate) fn new(string: &'term_string TermString) -> Self {
        let mut indices = string.as_str().grapheme_indices(true);
        let prev_index =
            indices.next().map_or(string.len(), |(index, _)| index);
        let next_index = string.len();

        Self { indices, prev_index, next_index, base: string.clone() }
    }
}

impl<'term_string> Iterator for TermStringIndices<'term_string> {
    type Item = (usize, TermGrapheme);

    fn next(&mut self) -> Option<Self::Item> {
        if self.prev_index == self.next_index {
            None
        } else {
            let index =
                self.indices.next().map_or(self.next_index, |(index, _)| index);
            let start = self.base.range.start + self.prev_index;
            let end = self.base.range.start + index;
            let alloc = self.base.alloc.clone();
            let term_string = TermString { alloc, range: start .. end };
            self.prev_index = index;
            Some((term_string.range.start, TermGrapheme { term_string }))
        }
    }
}

impl<'term_string> DoubleEndedIterator for TermStringIndices<'term_string> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.prev_index == self.next_index {
            None
        } else {
            let index = self
                .indices
                .next_back()
                .map_or(self.prev_index, |(index, _)| index);
            let start = self.base.range.start + index;
            let end = self.base.range.start + self.next_index;
            let alloc = self.base.alloc.clone();
            let term_string = TermString { alloc, range: start .. end };
            self.next_index = index;
            Some((term_string.range.start, TermGrapheme { term_string }))
        }
    }
}

impl<'term_string> fmt::Debug for TermStringIndices<'term_string> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("TermStringIndices")
            .field("base", &self.base)
            .field("prev_index", &self.prev_index)
            .field("next_index", &self.next_index)
            .finish()
    }
}

/// Iterator only over the grapheme cluster ([`TermGrapheme`])s of a
/// `TermString`.
#[derive(Debug)]
pub struct TermStringIter<'term_string> {
    inner: TermStringIndices<'term_string>,
}

impl<'term_string> Iterator for TermStringIter<'term_string> {
    type Item = TermGrapheme;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, grapheme)| grapheme)
    }
}

impl<'term_string> DoubleEndedIterator for TermStringIter<'term_string> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(_, grapheme)| grapheme)
    }
}

impl<'term_string> IntoIterator for &'term_string TermString {
    type Item = TermGrapheme;
    type IntoIter = TermStringIter<'term_string>;

    fn into_iter(self) -> Self::IntoIter {
        TermStringIter { inner: self.indices() }
    }
}
