//! This module provides a string type used to be printed to the terminal. It is
//! called "[`TermString`]". You can use the macro [`tstring!`] to build a
//! [`TermString`] as a shortcut for [`TermString::new_lossy`].
//!
//! This module also provides a type [`TermGrapheme`], which corresponds to what
//! a human sees as a single character ("grapheme cluster"). It might be
//! composed of a single unicode codepoint or of several. However,
//! [`TermGrapheme`]s made of several characters are not portable, as Windows'
//! `cmd` by default prints the multiple characters of a grapheme separatedly,
//! while most Linux ANSI terminals will print them together if they form a
//! single grapheme cluster.
//!
//! To concat [`TermString`]s and [`TermGrapheme`]s together you can use the
//! macro [`tstring_concat!`].

mod index;
mod iter;
mod error;

#[cfg(test)]
mod test;

pub use self::{
    error::{
        DiacriticAtStart,
        InvalidControl,
        NotAGrapheme,
        TermGraphemeError,
        TermStringError,
    },
    index::Index,
    iter::{TermStringIndices, TermStringIter},
};

use lazy_static::lazy_static;
use std::{
    cmp::Ordering,
    convert::TryFrom,
    fmt,
    hash::{Hash, Hasher},
    iter::FromIterator,
    ops::{Deref, Range},
    path::Path,
    sync::Arc,
};
use unicode_segmentation::UnicodeSegmentation;

/// Graphical string: a string valid to be printed on a terminal for graphic
/// purpouse.
#[derive(Debug, Clone)]
pub struct TermString {
    alloc: Arc<str>,
    range: Range<usize>,
}

impl TermString {
    /// Builds a new graphical string.
    ///
    /// The string must not start with a diacritic character. Diacritic here is
    /// not "^" or "~", but rather a diacritic that when inserted combines with
    /// the previous character. Like the tilde in "ỹ" which can be separated
    /// from "y". On the other hand, the combination "ỹ" is valid and forms a
    /// single grapheme cluster. The diacritic is only invalid when separated.
    ///
    /// Control characters also trigger an error, because those would allow the
    /// terminal to be controlled.
    pub fn new<S>(string: S) -> Result<Self, TermStringError>
    where
        S: AsRef<str>,
    {
        for (position, ch) in string.as_ref().char_indices() {
            if ch.is_control() {
                Err(InvalidControl { position })?;
            }
        }

        let mut new_string = string.as_ref().to_owned();
        new_string.replace_range(0 .. 0, "a");
        let mut iter = new_string.grapheme_indices(true);
        iter.next();
        let index = iter.next().map_or(new_string.len(), |(index, _)| index);
        if index != 1 {
            Err(DiacriticAtStart)?;
        }
        new_string.replace_range(0 .. 1, "");

        let range = 0 .. new_string.len();
        Ok(TermString { alloc: new_string.into(), range })
    }

    /// Creates a new [`TermString`], but replaces error with the replacement
    /// character "�".
    pub fn new_lossy<S>(string: S) -> Self
    where
        S: AsRef<str>,
    {
        let mut new_string = String::new();
        for ch in string.as_ref().chars() {
            new_string.push(if ch.is_control() { '�' } else { ch });
        }

        new_string.replace_range(0 .. 0, "a");
        let mut iter = new_string.grapheme_indices(true);
        iter.next();
        let index = iter.next().map_or(new_string.len(), |(index, _)| index);
        let replacement = if index != 1 { "�" } else { "" };
        new_string.replace_range(0 .. 1, replacement);

        let range = 0 .. new_string.len();
        TermString { alloc: new_string.into(), range }
    }

    /// Counts how many grapheme clusters the string contains by iterating the
    /// string.
    pub fn count_graphemes(&self) -> usize {
        self.as_str().graphemes(true).count()
    }

    /// Converts into a reference to a plain string.
    pub fn as_str(&self) -> &str {
        &self.alloc[self.range.clone()]
    }

    /// Indexes the string by returning `None` if out of bounds. `usize` will
    /// return [`TermGrapheme`]s, ranges will return sub-[`TermString`]s.
    /// WARNING: this method is, prefere iterating instead.
    pub fn get<I>(&self, index: I) -> Option<I::Output>
    where
        I: Index,
    {
        index.get(self)
    }

    /// Indexes the string by panicking if out of bounds. `usize` will
    /// return [`TermGrapheme`]s, ranges will return sub-[`TermString`]s.
    /// WARNING: this method is slow, prefere iterating instead.
    pub fn index<I>(&self, index: I) -> I::Output
    where
        I: Index,
    {
        index.index(self)
    }

    /// Iterates over indices and grapheme clusters of this string.
    pub fn indices(&self) -> TermStringIndices {
        TermStringIndices::new(self)
    }

    /// Iterates only over graphemes of this string.
    pub fn iter(&self) -> TermStringIter {
        self.into_iter()
    }

    /// De-slices a sub-[`TermString`] into the original buffer.
    pub fn full_buf(self) -> Self {
        Self { alloc: self.alloc.clone(), range: 0 .. self.alloc.len() }
    }
}

impl FromIterator<TermGrapheme> for TermString {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = TermGrapheme>,
    {
        let mut buf = String::new();
        for grapheme in iter {
            buf.push_str(grapheme.as_str());
        }
        let range = 0 .. buf.len();
        Self { alloc: buf.into(), range }
    }
}

impl FromIterator<TermString> for TermString {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = TermString>,
    {
        let mut buf = String::new();
        for gstr in iter {
            buf.push_str(gstr.as_str());
        }
        let range = 0 .. buf.len();
        Self { alloc: buf.into(), range }
    }
}

impl<'buf> FromIterator<&'buf TermGrapheme> for TermString {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'buf TermGrapheme>,
    {
        let mut buf = String::new();
        for grapheme in iter {
            buf.push_str(grapheme.as_str());
        }
        let range = 0 .. buf.len();
        Self { alloc: buf.into(), range }
    }
}

impl<'buf> FromIterator<&'buf TermString> for TermString {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'buf TermString>,
    {
        let mut buf = String::new();
        for gstr in iter {
            buf.push_str(gstr.as_str());
        }
        let range = 0 .. buf.len();
        Self { alloc: buf.into(), range }
    }
}

impl<'buf> FromIterator<StringOrGraphm<'buf>> for TermString {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = StringOrGraphm<'buf>>,
    {
        // also a heuristics
        let mut buf = String::with_capacity(80);
        for gstr in iter {
            buf.push_str(gstr.as_str());
        }
        let range = 0 .. buf.len();
        Self { alloc: buf.into(), range }
    }
}

impl Default for TermString {
    fn default() -> Self {
        Self { alloc: Arc::from(""), range: 0 .. 0 }
    }
}

impl AsRef<str> for TermString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for TermString {
    fn as_ref(&self) -> &Path {
        self.as_str().as_ref()
    }
}

impl Deref for TermString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl PartialEq for TermString {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.alloc, &other.alloc) && self.range == other.range
            || self.as_str() == other.as_str()
    }
}

impl Eq for TermString {}

impl PartialOrd for TermString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for TermString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for TermString {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_str().hash(state)
    }
}

impl fmt::Display for TermString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl<'buf> TryFrom<&'buf str> for TermString {
    type Error = TermStringError;

    fn try_from(buf: &'buf str) -> Result<Self, Self::Error> {
        Self::new(buf)
    }
}

impl TryFrom<String> for TermString {
    type Error = TermStringError;

    fn try_from(buf: String) -> Result<Self, Self::Error> {
        Self::new(buf)
    }
}

/// A grapheme cluster. Represents what a human visually sees as a character.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TermGrapheme {
    tstring: TermString,
}

impl Default for TermGrapheme {
    /// Returns the grapheme for the space " ".
    fn default() -> Self {
        lazy_static! {
            static ref DEFAULT_GRAPHM: Arc<str> = Arc::from(" ");
        }
        let alloc = DEFAULT_GRAPHM.clone();
        let range = 0 .. alloc.len();
        Self { tstring: TermString { alloc, range } }
    }
}

impl fmt::Display for TermGrapheme {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.tstring)
    }
}

impl Deref for TermGrapheme {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl TermGrapheme {
    /// Builds a new grapheme cluster. The argument must be composed of only one
    /// grapheme.
    ///
    /// The string must not start with a diacritic character.
    /// Diacritic here is not "^" or "~", but rather a diacritic that when
    /// inserted combines with the previous character. Like the tilde in "ỹ"
    /// which can be separated from "y". On the other hand, the combination "ỹ"
    /// is valid and forms a single grapheme. The diacritic is only invalid when
    /// separated.
    ///
    /// Control characters also trigger an error, because those would allow the
    /// terminal to be controlled.
    pub fn new<S>(string: S) -> Result<Self, TermGraphemeError>
    where
        S: AsRef<str> + Into<String>,
    {
        let first = string
            .as_ref()
            .graphemes(true)
            .next()
            .ok_or_else(|| NotAGrapheme)?;
        if first.len() != string.as_ref().len() {
            Err(NotAGrapheme)?;
        }

        for (position, ch) in string.as_ref().char_indices() {
            if ch.is_control() {
                Err(InvalidControl { position })?;
            }
        }

        let mut new_string = string.into();
        new_string.replace_range(0 .. 0, "a");
        let count = new_string.graphemes(true).count();
        if count == 1 {
            Err(DiacriticAtStart)?;
        }
        new_string.replace_range(0 .. 1, "");

        let range = 0 .. new_string.len();
        let tstring = TermString { alloc: new_string.into(), range };
        Ok(Self { tstring })
    }

    /// Creates a new [`TermGrapheme`], but replaces error with the replacement
    /// character "�". Truncates the string it contains more than one grapheme.
    pub fn new_lossy<S>(string: S) -> Self
    where
        S: Into<String> + AsRef<str>,
    {
        let actual_str = string.as_ref().graphemes(true).next().unwrap_or("�");
        let mut new_string = String::new();
        for ch in actual_str.chars() {
            new_string.push(if ch.is_control() { '�' } else { ch });
        }

        new_string.replace_range(0 .. 0, "a");
        let mut iter = new_string.grapheme_indices(true);
        iter.next();
        let index = iter.next().map_or(new_string.len(), |(index, _)| index);
        let replacement = if index != 1 { "�" } else { "" };
        new_string.replace_range(0 .. 1, replacement);

        let range = 0 .. new_string.len();
        let tstring = TermString { alloc: new_string.into(), range };
        Self { tstring }
    }

    /// Returns the grapheme for the space " ". This is the default grapheme,
    /// used in `Default`.
    pub fn space() -> TermGrapheme {
        Self::default()
    }

    /// Converts into a reference of a plain string.
    pub fn as_str(&self) -> &str {
        &self.tstring
    }

    /// Returns the underlying string buffer of this [`TermGrapheme`].
    pub fn as_tstring(&self) -> &TermString {
        &self.tstring
    }

    /// Converts into the underlying string buffer of this [`TermGrapheme`].
    pub fn into_tstring(self) -> TermString {
        self.tstring
    }
}

impl<'buf> TryFrom<&'buf str> for TermGrapheme {
    type Error = TermGraphemeError;

    fn try_from(buf: &'buf str) -> Result<Self, Self::Error> {
        Self::new(buf)
    }
}

impl TryFrom<String> for TermGrapheme {
    type Error = TermGraphemeError;

    fn try_from(buf: String) -> Result<Self, Self::Error> {
        Self::new(buf)
    }
}

/// Either a string or a grapheme reference. Used by [`tstring_concat!`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StringOrGraphm<'buf> {
    /// A terminal grapheme cluster.
    Graphm(&'buf TermGrapheme),
    /// A terminal string.
    String(&'buf TermString),
}

impl<'buf> StringOrGraphm<'buf> {
    /// The inherent string under this [`TermString`] or this [`TermGrapheme`].
    pub fn as_str(self) -> &'buf str {
        match self {
            StringOrGraphm::Graphm(grapheme) => grapheme.as_str(),
            StringOrGraphm::String(gstr) => gstr.as_str(),
        }
    }

    /// If it this is a [`TermString`], return it; if this is a
    /// [`TermGrapheme`], get the inherent [`TermString`].
    pub fn as_tstring(self) -> &'buf TermString {
        match self {
            StringOrGraphm::Graphm(grapheme) => grapheme.as_tstring(),
            StringOrGraphm::String(gstr) => gstr,
        }
    }
}

impl<'buf> AsRef<str> for StringOrGraphm<'buf> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'buf> AsRef<TermString> for StringOrGraphm<'buf> {
    fn as_ref(&self) -> &TermString {
        self.as_tstring()
    }
}

impl<'buf> Deref for StringOrGraphm<'buf> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.as_tstring()
    }
}

impl<'buf> From<&'buf TermGrapheme> for StringOrGraphm<'buf> {
    fn from(grapheme: &'buf TermGrapheme) -> StringOrGraphm<'buf> {
        StringOrGraphm::Graphm(grapheme)
    }
}

impl<'buf> From<&'buf TermString> for StringOrGraphm<'buf> {
    fn from(tstring: &'buf TermString) -> StringOrGraphm<'buf> {
        StringOrGraphm::String(tstring)
    }
}
