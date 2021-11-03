#![feature(proc_macro_hygiene, decl_macro, concat_idents)]

use app_model::jwt::*;
use rocket::{
	http::*,
	request::FromParam,
	response::*,
	routes,
	get,
};
use define_api::rest_handlers;
use anyhow::anyhow;
use std::io::Result;
use std::path::Path;
use std::str::FromStr;

struct SerdeParam<T>(T)
where
	T: FromStr;

impl<T> From<T> for SerdeParam<T>
where
	T: FromStr,
{
	fn from(o: T) -> Self {
		Self(o)
	}
}
impl<T> std::ops::Deref for SerdeParam<T>
where
	T: FromStr,
{
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<'r, T> FromParam<'r> for SerdeParam<T>
where
	T: FromStr,
	<T as FromStr>::Err: std::fmt::Display,
{
	type Error = anyhow::Error;
	fn from_param(param: &'r RawStr) -> std::result::Result<Self, Self::Error> {
		T::from_str(param.as_str())
			.map(|t: T| Self::from(t))
			.map_err(|e| anyhow!(format!("Failed to parse \'{}\': {}", param, e)))
	}
}

pub fn get_file<P: AsRef<Path>>(path: P) -> Result<NamedFile> {
	NamedFile::open(path)
}
static CLIENT_DIR: &'static str = "../client";

#[get("/<app>")]
fn get_html(app: &RawStr) -> Result<NamedFile> {
	let _ = app;
	get_file(format!("./{}/app.html", CLIENT_DIR))
}
#[get("/")]
fn get_root_html() -> Result<NamedFile> {
	get_html("".into())
}
#[get("/users/<id>")]
fn user_page(id: &RawStr) -> Result<NamedFile> {
	let _ = id;
	get_file(format!("./{}/app.html", CLIENT_DIR))
}
#[get("/projects/<id>")]
fn project_page(id: &RawStr) -> Result<NamedFile> {
	let _ = id;
	get_file(format!("./{}/app.html", CLIENT_DIR))
}
#[get("/tasks/<id>")]
fn task_page(id: &RawStr) -> Result<NamedFile> {
	let _ = id;
	get_file(format!("./{}/app.html", CLIENT_DIR))
}

#[get("/<dir>/styles/<file_name>")]
fn get_style_css(dir: &RawStr, file_name: &RawStr) -> Result<NamedFile> {
	get_file(format!("./{}/styles/{}", dir, file_name))
}
#[get("/pkg/<file_name>")]
fn get_pkg_js(file_name: &RawStr) -> Result<NamedFile> {
	get_file(format!("./{}/pkg/{}", CLIENT_DIR, file_name))
}
#[get("/img/<file_name>")]
fn get_img_file(file_name: &RawStr) -> Result<NamedFile> {
	get_file(format!("./img/{}", file_name))
}

#[get("/api/token_valid")]
fn token_valid(token: JWT) {
	let _ = token;
}
fn main() {
	rocket::custom(
		rocket::Config::build(rocket::config::Environment::Staging)
			.address("0.0.0.0")
			.port(8000)
			.finalize()
			.unwrap(),
	)
	.mount(
		"/",
		vec![
			routes![
				get_root_html,
				get_html,
				user_page,
				project_page,
				task_page,
				token_valid,
				get_style_css,
				get_pkg_js,
				get_img_file,
				api::login,
				api::register,
				api::handlers::get_user_projects,
				api::handlers::get_project_tasks,
				//api::handlers::project_create_subtask,
				api::handlers::interpret_text,
				api::handlers::query_text,
			],
			rest_handlers!(Task),
			rest_handlers!(Project),
			rest_handlers!(User),
		]
		.concat(),
	)
	.launch();
}
