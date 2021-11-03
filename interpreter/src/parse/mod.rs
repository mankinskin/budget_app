pub use nom::{
	character::{
		complete::*,
		*,
	},
	combinator::*,
	error::*,
	*,
};

mod chrono;
mod error;
mod units;
pub use ::chrono::*;
pub use units::*;

pub trait Parse<'a>: Sized {
	fn parse(input: &'a str) -> IResult<&'a str, Self>;
}

pub enum Action {
	Give,
	Get,
}

impl<'a> Parse<'a> for Action {
	named!(
		parse(&'a str) -> Self,
		alt!(
		tag_no_case!("have gotten") => { |_| Self::Get } |
		tag_no_case!("got") => { |_| Self::Get } |
		tag_no_case!("get") => { |_| Self::Get } |
		tag_no_case!("will get") => { |_| Self::Get } |
		tag_no_case!("have given") => { |_| Self::Give } |
		tag_no_case!("gave") => { |_| Self::Give } |
		tag_no_case!("give") => { |_| Self::Give } |
		tag_no_case!("will give") => { |_| Self::Give }
		)
	);
}

mod tests {
	use super::*;
}
