use super::*;
use serde::{
	Serialize,
	Deserialize,
};
#[derive(PartialEq, Eq, Clone, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum TextElement {
	Word(Word),
	Punctuation(Punctuation),
	Start,
	Stop,
	Empty,
}
impl TextElement {
	pub fn is_stop(&self) -> bool {
		match self {
			TextElement::Punctuation(Punctuation::Dot) => true,
			TextElement::Punctuation(Punctuation::QuestionMark) => true,
			TextElement::Punctuation(Punctuation::ExclamationMark) => true,
			TextElement::Punctuation(Punctuation::DoubleQuote) => true,
			TextElement::Punctuation(Punctuation::Quote) => true,
			TextElement::Start => false,
			TextElement::Stop => true,
			TextElement::Empty => true,
			_ => false
		}
	}
}
impl From<Word> for TextElement {
	fn from(w: Word) -> Self {
		Self::Word(w)
	}
}
impl From<Punctuation> for TextElement {
	fn from(p: Punctuation) -> Self {
		Self::Punctuation(p)
	}
}
use std::fmt::{Debug, Display, self};
impl Debug for TextElement {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self)
	}
}
impl Display for TextElement {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Word(w) => write!(f, "{}", w),
			Self::Punctuation(p) => write!(f, "{}", p),
			Self::Empty => write!(f, "*"),
			Self::Start => write!(f, ">>"),
			Self::Stop => write!(f, ";"),
		}
	}
}
use crate::parse::*;
impl<'a> Parse<'a> for TextElement {
	named!(
		parse(&'a str) -> Self,
		alt!(
			map!(
				Word::parse,
				|s| Self::from(s)
			) |
			map!(
				Punctuation::parse,
				|s| Self::from(s)
			)
		)
	);
}

mod tests {
	#[allow(unused)]
	use super::*;
	#[test]
	fn parse_textelement() {
		assert_eq!(TextElement::parse(",").unwrap().1, TextElement::Punctuation(Punctuation::Comma));
		assert_eq!(TextElement::parse(".").unwrap().1, TextElement::Punctuation(Punctuation::Dot));
		assert_eq!(TextElement::parse(":").unwrap().1, TextElement::Punctuation(Punctuation::Colon));
		assert_eq!(TextElement::parse(";").unwrap().1, TextElement::Punctuation(Punctuation::Semicolon));
		assert_eq!(TextElement::parse("\'").unwrap().1, TextElement::Punctuation(Punctuation::Quote));
		assert_eq!(TextElement::parse("\"").unwrap().1, TextElement::Punctuation(Punctuation::DoubleQuote));
		let words = vec![
			"Hello",
			"Hi",
			"yes",
			"aha",
			"Mathematik",
			"mathmatical"
		];
		for word in words {
			assert_eq!(TextElement::parse(word).unwrap().1, TextElement::Word(Word::from(word)));
		}
	}
}
