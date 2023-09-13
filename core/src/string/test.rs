use crate::string::{TermGrapheme, TermString};

#[test]
fn valid_grapheme() {
    TermGrapheme::new("a").unwrap();
    TermGrapheme::new("ç").unwrap();
    TermGrapheme::new("ỹ").unwrap();
    TermGrapheme::new(" ").unwrap();
    TermGrapheme::new("ẽ̞").unwrap();
}

#[test]
fn invalid_grapheme() {
    TermGrapheme::new("\u{31e}").unwrap_err();
    TermGrapheme::new("").unwrap_err();
    TermGrapheme::new("abcde").unwrap_err();
    TermGrapheme::new("\n").unwrap_err();
}

#[test]
fn lossy_grapheme() {
    assert_eq!(TermGrapheme::new_lossy("a").as_str(), "a");
    assert_eq!(TermGrapheme::new_lossy("ç").as_str(), "ç");
    assert_eq!(TermGrapheme::new_lossy("ỹ").as_str(), "ỹ");
    assert_eq!(TermGrapheme::new_lossy(" ").as_str(), " ");
    assert_eq!(TermGrapheme::new_lossy("ẽ̞").as_str(), "ẽ̞");
    assert_eq!(TermGrapheme::new_lossy("\u{31e}").as_str(), "�̞");
    assert_eq!(TermGrapheme::new_lossy("").as_str(), "�");
    assert_eq!(TermGrapheme::new_lossy("abcde").as_str(), "a");
    assert_eq!(TermGrapheme::new_lossy("\n").as_str(), "�");
}

#[test]
fn valid_tstring() {
    TermString::new("abc").unwrap();
    TermString::new("çedilha").unwrap();
    TermString::new("ỹ").unwrap();
    TermString::new(" ").unwrap();
    TermString::new("ẽ̞").unwrap();
}

#[test]
fn invalid_tstring() {
    TermString::new("\u{31e}abc").unwrap_err();
    TermGrapheme::new("aa\n").unwrap_err();
}

#[test]
fn lossy_tstring() {
    assert_eq!(TermString::new_lossy("a").as_str(), "a");
    assert_eq!(TermString::new_lossy("ç").as_str(), "ç");
    assert_eq!(TermString::new_lossy("ỹ").as_str(), "ỹ");
    assert_eq!(TermString::new_lossy(" ").as_str(), " ");
    assert_eq!(TermString::new_lossy("ẽ̞").as_str(), "ẽ̞");
    assert_eq!(TermString::new_lossy("\u{31e}").as_str(), "�̞");
    assert_eq!(TermString::new_lossy("").as_str(), "");
    assert_eq!(TermString::new_lossy("abcde").as_str(), "abcde");
}

#[test]
fn indices_iter() {
    let string = tstring!["abćdef̴"];
    let mut iter = string.indices();
    assert_eq!(iter.next().unwrap(), (0, TermGrapheme::new("a").unwrap()));
    assert_eq!(iter.next().unwrap(), (1, TermGrapheme::new("b").unwrap()));
    assert_eq!(iter.next().unwrap(), (2, TermGrapheme::new("ć").unwrap()));
    assert_eq!(iter.next().unwrap(), (5, TermGrapheme::new("d").unwrap()));
    assert_eq!(iter.next_back().unwrap(), (7, TermGrapheme::new("f̴").unwrap()));
    assert_eq!(iter.next_back().unwrap(), (6, TermGrapheme::new("e").unwrap()));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}
