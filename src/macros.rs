/// Creates a `TermString` from string literal. Panicks if the string is
/// invalid.
#[macro_export]
macro_rules! tstring {
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
