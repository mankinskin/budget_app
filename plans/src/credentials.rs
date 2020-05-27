use crate::{
    user::*,
};
use updatable::{
    *,
};
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Updatable,
    Serialize,
    Deserialize,
    )]
pub struct Credentials {
    pub username: String,
    pub password: String,
}
impl Credentials {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn username_is_valid(&self) -> bool {
        self.username_invalid_text().is_empty()
    }
    pub fn password_is_valid(&self) -> bool {
        self.password_invalid_text().is_empty()
    }
    pub fn username_invalid_text(&self) -> String {
        match self.username.len() {
            0 | 8..=16 => String::new(),
            _ => String::from(
                "Username must be between 8 and 16 characters long."
            )
        }
    }
    pub fn password_invalid_text(&self) -> String {
        match self.password.len() {
            0 | 8..=16 => String::new(),
            _ => String::from(
                "Password must be between 8 and 16 characters long."
            )
        }
    }
}
impl From<&User> for Credentials {
    fn from(user: &User) -> Self {
        Self {
            username: user.name().clone(),
            password: user.password().clone(),
        }
    }
}
impl From<User> for Credentials {
    fn from(user: User) -> Self {
        Self {
            username: user.name().clone(),
            password: user.password().clone(),
        }
    }
}

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Updatable,
    Serialize,
    Deserialize,
    )]
pub struct AccessToken(String);

impl From<String> for AccessToken {
    fn from(s: String) -> Self {
        Self(s)
    }
}
