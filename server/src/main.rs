#[macro_use] extern crate rouille;
extern crate chrono;
extern crate colored;
extern crate serde_json;
extern crate plans;
use rouille::{
    Request,
    Response,
    input::json::*,
};
use chrono::{
    Utc,
};
use std::{
    fs::{
        File,
    },
    path::{
        Path,
    },
};
use colored::*;
use plans::{
    user::*,
    note::*,
    task::*,
};
fn log_request(r: &Request) {
    print!("[{}] {} {} {} ",
        Utc::now().format("%d.%m.%Y %T"),
        r.remote_addr().to_string().blue(),
        r.method(),
        r.raw_url());
}
fn log_response(r: &Response) {
    println!("{}", (|s: String| if r.is_error() {
        s.red()
    } else {
        s.green()
    })(r.status_code.to_string()));
}
fn get_html<P: AsRef<Path>>(path: P) -> Response {
    match File::open(path) {
        Ok(file) => Response::from_file("text/html", file),
        Err(e) => Response::text(e.to_string()),
    }
}
fn get_user(req: &Request) -> Response {
    let user = User::new("Server");
    rouille::Response::json(&user)
}
fn post_note(req: &Request) -> Response {
    let note: Note = try_or_400!(json_input(req));
    println!("Got note: {:#?}", note);
    rouille::Response::empty_204()
}
fn get_task(req: &Request) -> Response {
    let task =
        TaskBuilder::default()
            .title("Server Task".into())
            .description("This is the top level task.".into())
            .assignees(vec!["Heinz".into(), "Kunigunde".into(), "Andreas".into()])
            .children(vec![
                        TaskBuilder::default()
                            .title("First Item".into())
                            .description("This is the first sub task.".into())
                            .assignees(vec!["Heinz".into(), "Kunigunde".into()])
                            .children(vec![
                                TaskBuilder::default()
                                    .title("Second Level".into())
                                    .description("This is a sub sub task.".into())
                                    .assignees(vec!["Heinz".into(), "Kunigunde".into()])
                                    .children(vec![ ])
                                    .build()
                                    .unwrap(),
                            ])
                            .build()
                            .unwrap(),
                    ])
            .build()
            .unwrap();
    rouille::Response::json(&task)
}
fn post_task(req: &Request) -> Response {
    let task: Task = try_or_400!(json_input(req));
    println!("Got task: {:#?}", task);
    rouille::Response::empty_204()
}

fn handle_request(request: &Request) -> Response {
    log_request(request);
    let response = router!(request,
        (GET) (/tasks/tools) => {
            get_html("./tasks/index.html")
        },
        (GET) (/tasks) => {
            get_html("./tasks/index.html")
        },
        (GET) (/budget) => {
            get_html("./home/index.html")
        },
        (GET) (/api/task) => {
            get_task(request)
        },
        (POST) (/api/task) => {
            post_task(request)
        },
        (GET) (/api/user) => {
            get_user(request)
        },
        (POST) (/api/note) => {
            post_note(request)
        },
        (GET) (/user) => {
            get_html("./home/index.html")
        },
        (GET) (/profile) => {
            get_html("./home/index.html")
        },
        (GET) (/note) => {
            get_html("./home/index.html")
        },
        (GET) (/) => {
            get_html("./home/index.html")
        },
        _ => rouille::match_assets(request, "./")
    );
    log_response(&response);
    response
}
fn main() {
    let address = "0.0.0.0:8000";
    println!("Serving on {}", address);
    rouille::start_server(address, handle_request)
}
