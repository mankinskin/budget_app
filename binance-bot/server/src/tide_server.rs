use crate::{
    *,
    binance::{
        binance,
        PriceHistoryRequest,
        BinanceActor,
    },
    subscriptions::SubscriptionsActor,
    telegram::TelegramActor,
};
use tide_tracing::{
    TraceMiddleware,
};
use async_std::net::SocketAddr;
use tide_rustls::TlsListener;
#[allow(unused)]
use tracing::{
    debug,
    info,
    error,
    warn,
    trace,
};
use tide_websockets::{
    Message,
    WebSocket,
};
use tide::{
    Endpoint,
    Request,
    Body,
    Response,
    Middleware,
};
use futures_util::{
    StreamExt,
    SinkExt,
};
use shared::{
    PriceSubscription,
};
use app_model::{
    user::User,
    auth::{
        Credentials,
        login,
        register,
    }
};
use chrono::Utc;

macro_rules! client_file {
    ($path:expr) => {
        |_: tide::Request<()>| async move {
            let body = tide::Body::from_file(format!("{}/{}", CLIENT_PATH, $path)).await?;
            Ok(tide::Response::from(body))
        }
    }
}
macro_rules! index {
    () => { client_file!("index.html") }
}
fn wss_middleware() -> impl Middleware<()> + Endpoint<()> {
    WebSocket::new(async move |_, ws| {
        let (sink, stream) = ws.split();
        let stream = stream.map(|msg| msg.map(|msg| msg.into_data()));
        let sink = sink.with(async move |msg| Ok(Message::from(msg)) as Result<_, tide_websockets::Error>);
        websocket::connection(stream, sink).await;
        Ok(())
    })
}
async fn wss_handler(request: Request<()>) -> tide::Result {
    debug!("WSS Request");
    wss_middleware()
        .call(request)
    .await
}
fn root() -> std::io::Result<tide::Server<()>> {
    let mut root = tide::new();
    root.at("/").get(client_file!("index.html"));
    root.at("/favicon.ico").get(client_file!("favicon.ico"));
    root.at("/").serve_dir(format!("{}/pkg", CLIENT_PATH))?;
    Ok(root)
}
async fn login_handler(mut req: Request<()>) -> tide::Result {
    let credentials: Credentials = req.body_json().await?;
    match login::<database::Schema>(credentials).await {
        Ok(session) => {
            req.session_mut()
                .insert("session", session)
                .map(|_| Response::new(200))
                .map_err(|e| tide::Error::from_str(500, e.to_string()))
        },
        Err(e) => Err(e)
    }
}
async fn logout_handler(mut req: Request<()>) -> tide::Result {
    req.session_mut().remove("session");
    Ok(Response::new(200))
}
async fn registration_handler(mut req: Request<()>) -> tide::Result {
    let user: User = req.body_json().await?;
    match register::<database::Schema>(user).await {
        Ok(_session) => Ok(Response::new(200)),
        Err(e) => Err(tide::Error::from_str(500, e.to_string()))
    }
}
async fn post_subscription_handler(mut req: Request<()>) -> tide::Result<Body> {
    let s: PriceSubscription = req.body_json().await?;
    let r: Result<_, String> = subscriptions::add_subscription(s).await
        .map_err(|e| e.to_string());
    let body = Body::from_json(&r)?;
    debug!("{:#?}", body);
    Ok(body)
}
async fn get_subscription_list_handler(_req: Request<()>) -> tide::Result<Body> {
    debug!("Get subscription list handler");
    let r: Result<_, String> = Ok(subscriptions::get_subscription_list().await);
    debug!("Result: {:?}", r);
    Ok(Body::from_json(&r)?)
}
async fn delete_subscription_handler(req: Request<()>) -> tide::Result<Body> {
    let id: rql::Id<PriceSubscription> = req.param("id")?.parse()?;
    let r = subscriptions::delete_subscription(id).await.map_err(|e| e.to_string());
    Ok(Body::from_json(&r)?)
}
async fn get_subscription_handler(req: Request<()>) -> tide::Result<Body> {
    let id: rql::Id<PriceSubscription> = req.param("id")?.parse()?;
    let r = subscriptions::get_subscription(id).await.map_err(|e| e.to_string());
    Ok(Body::from_json(&r)?)
}
fn subscriptions_api() -> std::io::Result<tide::Server<()>> {
    let mut api = tide::new();
    api.at("/")
        .get(get_subscription_list_handler)
        .post(post_subscription_handler)
        .delete(delete_subscription_handler);
    api.at("/:id").get(get_subscription_handler);
    Ok(api)
}
fn auth_api() -> std::io::Result<tide::Server<()>> {
    let mut api = tide::new();
    api.at("/login").post(login_handler);
    api.at("/register").post(registration_handler);
    api.at("/logout").post(logout_handler);
    Ok(api)
}
fn api() -> std::io::Result<tide::Server<()>> {
    let mut api = tide::new();
    api.at("/auth").nest(auth_api()?);
    api.at("/subscriptions").nest(subscriptions_api()?);
    api.at("/price_history").nest(price_api()?);
    Ok(api)
}
async fn price_history_handler(_: Request<()>) -> tide::Result<Body> {
    match binance().await.get_symbol_price_history(PriceHistoryRequest {
            market_pair: "SOLBTC".into(),
            interval: Some(openlimits::model::Interval::OneHour),
            paginator: None,
        })
        .await
    {
        Ok(data) => Ok(Body::from_json(&data)?),
        Err(e) => Err(tide::Error::from_str(500, e.to_string()))
    }
}
fn price_api() -> std::io::Result<tide::Server<()>> {
    let mut api = tide::new();
    api.at("/").get(price_history_handler);
    Ok(api)
}

fn session_middleware() -> tide::sessions::SessionMiddleware<tide::sessions::MemoryStore> {
    tide::sessions::SessionMiddleware::new(
        tide::sessions::MemoryStore::new(),
        session::generate_secret().as_bytes()
    )
    .with_cookie_name("session")
    .with_session_ttl(Some(std::time::Duration::from_secs(session::EXPIRATION_SECS as u64)))
}

pub async fn run() -> std::io::Result<()> {
    let _subscription_actor = crate::actor_sys_mut().await.actor_of::<SubscriptionsActor>("subscriptions-actor").unwrap();
    let _telegram_actor = actor_sys_mut().await.actor_of::<TelegramActor>("telegram-actor").unwrap();
    let _binance_actor = actor_sys_mut().await.actor_of::<BinanceActor>("binance-actor").unwrap();

    let mut server = tide::new();
    server.with(TraceMiddleware::new());
    server.with(session_middleware());
    server.with(tide::utils::Before(async move |mut request: Request<()>| {
        let session = request.session_mut();
        if let Some(expiry) = session.expiry() {
            // time since expiry or (negative) until
            let dt = (Utc::now() - *expiry).num_seconds();
            if dt >= session::STALE_SECS as i64 {
                // expired and stale
                session.destroy()
            } else if dt >= 0 {
                // expired and not stale
                session.regenerate()
            }
        }
        request
    }));
    server.at("/").nest(root()?);
    server.at("/api").nest(api()?);
    server.at("/subscriptions").get(index!());
    server.at("/login").get(index!());
    server.at("/register").get(index!());
    server.at("/wss").get(wss_handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    server.listen(TlsListener::build()
            .addrs(addr)
            .cert(keys::to_key_path("tls.crt"))
            .key(keys::to_key_path("tls.key")),
        ).await?;
    Ok(())
}
