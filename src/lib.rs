pub mod editor;
pub mod entry;
pub mod preview;
pub use editor::{
    Edit,
    Editor,
};
pub use preview::{
    Previewable,
};
pub mod list;
pub use list::{
    List,
};
pub mod newdata;
pub mod remote;

use seed::prelude::*;

use std::fmt::Debug;
pub trait Init<Cfg>: Component {
    fn init(config: Cfg, orders: &mut impl Orders<<Self as Component>::Msg>) -> Self;
}
impl<Cfg, Cmp> Init<Cfg> for Cmp
where
    Self: Component,
    Cfg: Into<Cmp>,
{
    fn init(config: Cfg, _orders: &mut impl Orders<<Self as Component>::Msg>) -> Self {
        config.into()
    }
}
pub trait ComponentMsg: Debug + Clone + 'static {}
impl<T: Debug + Clone + 'static> ComponentMsg for T {}
pub trait Component {
    type Msg: ComponentMsg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>);
}
pub trait Viewable: Component {
    fn view(&self) -> Node<Self::Msg>;
}
