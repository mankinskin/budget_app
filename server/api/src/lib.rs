#![feature(proc_macro_hygiene, decl_macro)]
#![allow(unused)]
#[macro_use] extern crate define_api;
#[macro_use] extern crate lazy_static;

#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate seed;
extern crate serde_json;
extern crate serde;

#[cfg(not(target_arch="wasm32"))]
extern crate jwt;

extern crate rql;
extern crate plans;
extern crate database;
extern crate updatable;
api! {
    use rql::{
        Id,
    };
    use plans::{
        project::{
            Project,
        },
        user::{
            User,
        },
        task::{
            Task,
        },
    };
    use database::{
        *,
    };
    fn find_user_projects(id: Id<User>) -> Vec<Entry<Project>> {
        Project::filter(|project| project.members().contains(&id))
    }
    rest_api!(User);
    rest_api!(Project);
    rest_api!(Task);
}
