use seed::{
    *,
    prelude::*,
};
use crate::{
    config::{
        Component,
        Child,
    },
    preview::{self},
};
use api::{
    TableItem,
};
use database::{
    Entry,
};
use std::result::Result;
#[derive(Clone)]
pub enum Msg<T: TableItem + Component> {
    Refresh,
    Refreshed(Result<Option<Entry<T>>, String>),

    Delete,
    Deleted(Result<Option<T>, String>),

    Update,
    Updated(Result<Option<T>, String>),

    Data(<T as Component>::Msg),
    Preview(Box<preview::Msg<T>>),
}
impl<T: TableItem + Component> Component for Entry<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Refresh => {
                orders.perform_cmd(
                    T::get(self.id).map(|res| Msg::Refreshed(res))
                );
            },
            Msg::Refreshed(res) => {
                match res {
                    Ok(r) =>
                        if let Some(entry) = r {
                            *self = entry;
                        },
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::Delete => {
                orders.perform_cmd(
                    T::delete(self.id).map(|res| Msg::Deleted(res))
                );
            },
            Msg::Deleted(res) => {
                match res {
                    Ok(r) => { seed::log(r); },
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::Update => {
                orders.perform_cmd(
                    <T as TableItem>::update(self.id, T::Update::from(self.data.clone()))
                        .map(|res| Msg::Updated(res))
                );
            },
            Msg::Updated(res) => {
                match res {
                    Ok(r) => { seed::log(r); },
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::Data(msg) => {
                self.data.update(msg.clone(), &mut orders.proxy(Msg::Data));
                T::parent_msg(msg).map(|msg| orders.send_msg(msg));
            },
            Msg::Preview(_) => {
            },
        }
    }
}
