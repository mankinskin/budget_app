use crate::{
	subscriptions::SubscriptionsActor,
	websocket,
};
use shared::{
	ClientMessage,
	ServerMessage,
	WebsocketCommand,
};
#[allow(unused)]
use tracing::{
	debug,
	error,
	info,
	trace,
};
use riker::actors::*;
use futures::{
	StreamExt,
	SinkExt,
	channel::mpsc::{
		Sender,
		Receiver,
		channel,
	},
	Stream,
	Sink,
};
use riker::actors::{
	Sender as RkSender,
	CreateError
};
use std::{
	fmt::{
		Debug,
		Display,
	},
	convert::{
		TryInto,
	},
	result::Result,
};

#[derive(Clone, Debug)]
pub enum Msg {
	SetSubscriptions(ActorRef<<SubscriptionsActor as Actor>::Msg>),
}
#[actor(WebsocketCommand, ServerMessage, ClientMessage, Msg)]
#[derive(Debug)]
pub struct Connection {
	id: usize,
	sender: Sender<ServerMessage>,
	subscriptions: Option<ActorRef<<SubscriptionsActor as Actor>::Msg>>,
}
impl Connection {
	pub fn actor_name(id: usize) -> String {
		format!("Connection_{}", id)
	}
	pub async fn create(sender: Sender<ServerMessage>) -> Result<ActorRef<<Connection as Actor>::Msg>, CreateError> {
		let id = websocket::new_connection_id();
		crate::actor_sys().await.actor_of_args::<Connection, _>(&Self::actor_name(id), (id, sender))
	}
}
impl Actor for Connection {
	type Msg = ConnectionMsg;
	fn pre_start(&mut self, ctx: &Context<Self::Msg>) {
		debug!("Starting connection actor");
		let myself = ctx.myself();
		let id = self.id.clone();
		ctx.run(async move {
			let actor = SubscriptionsActor::create(id, myself.clone())
				.await
				.expect("Failed to create SubscriptionsActor!");
			myself.tell(Msg::SetSubscriptions(actor), None);
		}).expect("Failed to run future!");
	}
	fn post_stop(&mut self) {
		debug!("Stopped connection actor");
	}
	fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: RkSender) {
		self.receive(ctx, msg, sender);
	}
}
impl Receive<WebsocketCommand> for Connection {
	type Msg = ConnectionMsg;
	fn receive(&mut self, ctx: &Context<Self::Msg>, msg: WebsocketCommand, sender: RkSender) {
		trace!("WebsocketCommand in Connection actor");
		//debug!("Received {:#?}", msg);
		match msg {
			WebsocketCommand::Close => ctx.stop(ctx.myself()),
			WebsocketCommand::ClientMessage(msg) => self.receive(ctx, msg, sender),
			_ => {}
		}
	}
}
impl Receive<Msg> for Connection {
	type Msg = ConnectionMsg;
	fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: Msg, _sender: RkSender) {
		match msg {
			Msg::SetSubscriptions(actor) => self.subscriptions = Some(actor),
		}
	}
}
impl Receive<ClientMessage> for Connection {
	type Msg = ConnectionMsg;
	fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: ClientMessage, sender: RkSender) {
		trace!("ClientMessage in Connection actor");
		match msg {
			ClientMessage::Subscriptions(req) => if let Some(actor) = &self.subscriptions {
				actor.tell(req, sender);
			} else {
				error!("SubscriptionsActor not initialized!");
			},
		}
	}
}
impl Receive<ServerMessage> for Connection {
	type Msg = ConnectionMsg;
	fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: ServerMessage, _sender: RkSender) {
		trace!("ServerMessage in Connection actor");
		self.sender.try_send(msg).unwrap()
	}
}
impl ActorFactoryArgs<(usize, Sender<ServerMessage>)> for Connection {
	fn create_args((id, sender): (usize, Sender<ServerMessage>)) -> Self {
		debug!("Creating Connection");
		Self {
			id,
			sender,
			subscriptions: None,
		}
	}
}
pub async fn poll_messages<E, M, Rx>(connection: ActorRef<<Connection as Actor>::Msg>, mut rx: Rx)
	where E: ToString + Send + Debug,
		  M: From<ServerMessage> + TryInto<ClientMessage> + Send + Debug,
		  <M as TryInto<ClientMessage>>::Error: Display,
		  Rx: Stream<Item=Result<M, E>> + Send + 'static + Unpin,
{
	while let Some(msg) = rx.next().await {
		//debug!("ClientMessage received: {:#?}", msg);
		// convert M to ClientMessage
		let res = msg
			.map_err(|e| e.to_string())
			.and_then(|m| m.try_into()
					  .map_err(|e| format!("Failed to parse ClientMessage: {}", e))
					  as Result<ClientMessage, String>)
			.map(|msg| WebsocketCommand::ClientMessage(msg));
		match res {
			Ok(msg) => {
				// forward messages to connection actor
				if let WebsocketCommand::Close = msg {
					// stop listener
					crate::actor_sys().await.stop(connection);
					break;
				} else {
					// handle message
					connection.tell(msg, None);
				}
			},
			Err(e) => error!("{}", e),
		}
	}
}
pub async fn send_messages<M, Tx>(receiver: Receiver<ServerMessage>, tx: Tx) -> Result<(), String>
	where M: From<ServerMessage> + Send + Debug,
		  Tx: Sink<M> + Send + 'static,
		  <Tx as Sink<M>>::Error: ToString,
{
	receiver
		.map(|msg: ServerMessage|
			Ok(M::from(msg))
		)
		// send messages through websocket sink
		.forward(tx.sink_map_err(|e| e.to_string()))
		.await
}
pub async fn connection<E, M, Rx, Tx>(rx: Rx, tx: Tx)
	where E: ToString + Send + Debug,
		  M: From<ServerMessage> + TryInto<ClientMessage> + Send + Debug,
		  <M as TryInto<ClientMessage>>::Error: Display,
		  Rx: Stream<Item=Result<M, E>> + Send + 'static + Unpin,
		  Tx: Sink<M> + Send + 'static,
		  <Tx as Sink<M>>::Error: ToString,
{
	// connection lasts for the duration of this async fn
	debug!("Starting websocket connection");
	const CHANNEL_BUFFER_SIZE: usize = 100;
	let (sender, receiver) = channel(CHANNEL_BUFFER_SIZE);

	// create a connection actor with a ServerMessage sender
	let connection = Connection::create(sender).await.unwrap();
	let connection2 = connection.clone();
	// spawn listener for websocket stream
	let ws_listener = async_std::task::spawn(async move {
		poll_messages(connection2, rx).await
	});
	send_messages(receiver, tx).await
		.expect("Failed to forward connection messages to websocket!");
	//// wait for ServerMessages from connection actor
	ws_listener.await;
	//async_std::task::sleep(std::time::Duration::from_secs(100)).await;
	debug!("Closing websocket connection");
	crate::actor_sys().await.stop(connection.clone());
}

