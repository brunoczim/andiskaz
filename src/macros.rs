/// Creates a [`TermString`](crate::string::TermString) from string literal.
/// Currently, invalid string get the unicode replacement character in their
/// invalid characters. However, implementation may change to panic in those
/// cases.
///
/// # Example
/// ```
/// use andiskaz::tstring;
/// use andiskaz::string::TermString;
///
/// // A string with many grapheme clusters and unicode special codepoints.
/// let this_winter = tstring!["ðɪs wɪ̃ɾ̃ɚ"];
///
/// assert_eq!(this_winter, TermString::new("ðɪs wɪ̃ɾ̃ɚ").unwrap());
/// assert_eq!(this_winter.as_str(), "ðɪs wɪ̃ɾ̃ɚ");
/// ```
#[macro_export]
macro_rules! tstring {
    [] => {
        $crate::string::TermString::default()
    };

    [$s:expr $(,)?] => {
        $crate::string::TermString::new_lossy($s)
    };

    ($fmt:expr, $fst:expr $(,$rest:expr)* $(,)?) => {
        $crate::tstring![format!($fmt, $fst $(, $rest)*)]
    };
}

/// Concatenates various [`TermString`](crate::string::TermString) or
/// [`TermString`](crate::string::TermString)-like into a new
/// [`TermString`](crate::string::TermString). It takes everything by reference,
/// and it is possible to mix types.
///
/// # Example
/// ```
/// use andiskaz::{tstring, tstring_concat};
/// use andiskaz::string::{TermGrapheme, TermString};
///
/// let tomatoes = tstring!["Totatoes"];
/// let space = TermGrapheme::space();
/// let are = tstring!["are"];
/// let good = tstring!["good"];
///
/// let together: TermString = tstring_concat![tomatoes, space, are, space, good];
///
/// assert_eq!(together, TermString::new("Totatoes are good").unwrap());
/// assert_eq!(together.as_str(), "Totatoes are good");
/// ```
#[macro_export]
macro_rules! tstring_concat {
    [$($elem:expr,)*]  => {{
        (&[$($crate::string::StringOrGraphm::from(&$elem),)*])
            .iter()
            .map(|&x| x)
            .collect::<$crate::string::TermString>()
    }};
    [$($elem:expr),+]  => {
        tstring_concat![$($elem,)*]
    };
}

/// Writes the given formatting expression into the file `debug.txt`.
///
/// # Example
/// ```no_run
/// use andiskaz::coord::Vec2;
///
/// let coords = Vec2 { x: 3, y: 5 };
/// andiskaz::tdebug!("coords = {:?}\n", coords);
/// ```
#[macro_export]
macro_rules! tdebug {
    ($($tok:tt)+) => {{
        use std::fs::OpenOptions;
        use std::io::Write;
        let mut file = OpenOptions::new()
            .append(true)
            .truncate(false)
            .create(true)
            .open("debug.txt")
            .unwrap();
        write!(file, $($tok)+).unwrap();
    }};
}
