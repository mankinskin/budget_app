use seed::{
    *,
    prelude::*,
};
use plans::{
    credentials::*,
    user::*,
};
use crate::{
    page,
    register,
    route::{
        Route,
    },
    root::{
        self,
        GMsg,
    },
};
/// credential input component
#[derive(Clone, Default)]
pub struct Model {
    credentials: Credentials,
}
impl Model {
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }
    pub fn empty() -> Self {
        Self::default()
    }
}
pub fn init(_config: Config, _orders: &mut impl Orders<Msg, GMsg>) -> Model {
    Model::empty()
}
#[derive(Clone, Default)]
pub struct Config {
}
#[derive(Clone)]
pub enum Msg {
    ChangeUsername(String),
    ChangePassword(String),
    LoginResponse(Result<UserSession, String>),
    Login,
    Register,
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeUsername(u) => model.credentials.username = u,
        Msg::ChangePassword(p) => model.credentials.password = p,
        Msg::Login => {
            seed::log!("Logging in...");
            orders.perform_cmd(
                api::login(model.credentials().clone())
                    .map(|result: Result<UserSession, FetchError>| {
                        Msg::LoginResponse(result.map_err(|e| format!("{:?}", e)))
                    })
            );
        },
        Msg::LoginResponse(result) => {
            match result {
                Ok(session) => {
                    orders.send_g_msg(root::GMsg::SetSession(session));
                    page::go_to(Route::Home, orders);
                },
                Err(e) => {seed::log!(e)}
            }
        },
        Msg::Register => {
            page::go_to(register::Config::default(), orders);
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    // login form
    form![
        // Username field
        label![
            "Username"
        ],
        input![
            attrs!{
                At::Placeholder => "Username",
                At::Value => model.credentials.username,
            },
            input_ev(Ev::Input, Msg::ChangeUsername)
        ],
        div![
            model.credentials.username_invalid_text()
        ],
        // Password field
        label![
            "Password"
        ],
        input![
            attrs!{
                At::Type => "password",
                At::Placeholder => "Password",
                At::Value => model.credentials.password,
            },
            input_ev(Ev::Input, Msg::ChangePassword)
        ],
        div![
            model.credentials.password_invalid_text()
        ],
        // Login Button
        button![
            attrs!{
                At::Type => "submit",
            },
            "Login"
        ],
        ev(Ev::Submit, |ev| {
            ev.prevent_default();
            Msg::Login
        }),
        // Register Button
        button![simple_ev(Ev::Click, Msg::Register), "Register"],
    ]
}
