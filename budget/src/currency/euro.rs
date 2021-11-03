use crate::currency::{
	Currency,
	Quantity,
	Units,
	Value,
};
use std::ops::{
	Add,
	AddAssign,
	Mul,
	MulAssign,
	Neg,
	Sub,
	SubAssign,
};

#[derive(Clone, Debug)]
pub struct Euro(pub Units);

use std::fmt;
impl fmt::Display for Euro {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}€", self.0)
	}
}

#[allow(unused)]
pub fn euro() -> Euro {
	Euro::from(1)
}
impl From<Euro> for Units {
	fn from(euro: Euro) -> Self {
		euro.0
	}
}
impl From<Units> for Euro {
	fn from(units: Units) -> Self {
		Euro(units)
	}
}
impl Quantity for Euro {
	fn amount(&self) -> Units {
		self.0
	}
}

impl Currency for Euro {
	fn unit_value() -> Value {
		1.0
	}
}
use std::cmp::Ordering;
impl PartialOrd for Euro {
	fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
		self.0.partial_cmp(&rhs.0)
	}
}
impl PartialEq for Euro {
	fn eq(&self, rhs: &Self) -> bool {
		self.0.eq(&rhs.0)
	}
}
impl Eq for Euro {}
impl Ord for Euro {
	fn cmp(&self, rhs: &Self) -> Ordering {
		self.0.cmp(&rhs.0)
	}
}
impl AddAssign for Euro {
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0
	}
}
impl SubAssign for Euro {
	fn sub_assign(&mut self, rhs: Self) {
		self.0 -= rhs.0
	}
}
impl MulAssign for Euro {
	fn mul_assign(&mut self, rhs: Self) {
		self.0 *= rhs.0
	}
}

impl Neg for Euro {
	type Output = Self;
	fn neg(self) -> Self::Output {
		Euro(-self.0)
	}
}
impl Mul for Euro {
	type Output = Self;
	fn mul(self, rhs: Self) -> Self::Output {
		Euro(rhs.0 * self.0)
	}
}
impl Mul<Units> for Euro {
	type Output = Self;
	fn mul(self, rhs: Units) -> Self::Output {
		Euro(rhs * self.0)
	}
}
impl Mul<Euro> for Units {
	type Output = Euro;
	fn mul(self, rhs: Euro) -> Self::Output {
		Euro(rhs.0 * self)
	}
}

impl Add for Euro {
	type Output = Self;
	fn add(self, rhs: Self) -> Self::Output {
		Euro(self.0 + rhs.0)
	}
}
impl Sub for Euro {
	type Output = Self;
	fn sub(self, rhs: Self) -> Self::Output {
		Euro(self.0 - rhs.0)
	}
}

use crate::interpreter::parse::*;

impl<'a> Parse<'a> for Euro {
	named!(
		parse(&'a str) -> Self,
		map!(
			alt!(
				preceded!(tag!("€"), Units::parse)
					| terminated!(Units::parse, tag!("€"))
					| complete!(terminated!(Units::parse, tag_no_case!(" Euros")))
					| terminated!(Units::parse, tag_no_case!(" Euro"))
			),
			|u| Euro::from(u)
		)
	);
}

mod tests {
	#[allow(unused)]
	use super::*;

	#[test]
	fn parse_euro() {
		crate::cartesian! {
			["{}€", "{} Euro", "€{}", "{} Euros"],
			[{1}, {32}, {1823}, {99999999}]
			($fmt:tt {$u:expr}) => {
				assert_eq!(
					Euro::parse(&format!($fmt, $u)).unwrap().1,
					Euro::from($u)
					);
			}
		}
	}
}
