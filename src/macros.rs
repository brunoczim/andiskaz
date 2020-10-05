/// Creates a `TermString` from string literal. Panicks if the string is
/// invalid.
#[macro_export]
macro_rules! term_string {
    [] => {
        $crate::string::TermString::default()
    };

    [$s:expr] => {
        $crate::string::TermString::new_lossy($s)
    };
}

/// Creates a `TermString` from various other `TermString`-like fragments by
/// concatenation.
#[macro_export]
macro_rules! term_string_concat {
    [$($elem:expr,)*]  => {{
        (&[$($crate::string::StringOrGraphm::from(&$elem),)*])
            .iter()
            .map(|&x| x)
            .collect::<$crate::string::TermString>()
    }};
    [$($elem:expr),+]  => {
        term_string_concat![$($elem,)*]
    };
}
