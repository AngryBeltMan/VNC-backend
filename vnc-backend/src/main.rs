use std::{
    net::SocketAddr,
    collections::HashMap,
};
use async_channel;
use futures::{stream::{self as future_stream,Stream}, StreamExt, SinkExt};
use async_mutex::Mutex;
use tower_http::cors::CorsLayer;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade},Path, State},
    response::{IntoResponse,sse::{Event,KeepAlive,Sse}},
    routing::{get,post},
    Router, http::{HeaderValue, Method},
};
use std::sync::Arc;
pub struct SharedState {
    sender:Mutex<HashMap<String,Arc<async_channel::Sender<Vec<u8>>>>>,
    receiver:Mutex<HashMap<String,Arc<Mutex<async_channel::Receiver<Vec<u8>>>>>>,
    keyboard_sender:Mutex<HashMap<String,Arc<async_channel::Sender<String>>>>,
    keyboard_receiver:Mutex<HashMap<String,Arc<Mutex<async_channel::Receiver<String>>>>>,
}
mod keyboard_ws;
mod keyboard_ws_client;
mod frames_ws;
mod delete_server;

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    // let (sender,receiver) = async_channel::bounded(1);
    let app_state = Arc::new(SharedState {
        sender:Mutex::new(HashMap::new()),
        receiver:Mutex::new(HashMap::new()),
        keyboard_sender:Mutex::new(HashMap::new()),
        keyboard_receiver:Mutex::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/",get(home))
        .route("/ws/:code",get(ws_handler))
        .route("/ws/frames/:code",get(frames_ws::frames_ws_handler))
        .route("/ws/keyboard/:code",get(keyboard_ws::keyboard_websocket_handler))
        .route("/ws/client/keyboard/:code",get(keyboard_ws_client::keyboard_ws_client_handler))
        .route("/delete/server/:code",get(delete_server::delete_server))
        .with_state(app_state)
        .layer(
            CorsLayer::new()
            .allow_origin("*".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET,Method::POST])
        );
    Ok(app.into())
    // let addr = SocketAddr::from(([127,0,0,1],8080));
    // println!("listening on address {}",&addr);
    // axum::Server::bind(&addr)
    //     .serve(app.into_make_service_with_connect_info::<SocketAddr>())
    //     .await.unwrap();
}
async fn home() -> String {
    String::from("version 0.1.0")
}
async fn ws_handler(
    ws:WebSocketUpgrade,
    Path(code): Path<String>,
    State(state):State<Arc<SharedState>>
    ) -> impl IntoResponse {
    if !state.sender.lock().await.contains_key(&code) {
        let (sender,reciever) = async_channel::bounded(1);
        state.sender.lock().await.insert(code.clone(), sender.into());
        state.receiver.lock().await.insert(code.clone(), Arc::new(Mutex::new(reciever)));
        println!("new ws handler inserted");
    }
    ws.on_upgrade(move |socket| handle_socket(socket,state,code))
}

async fn handle_socket(mut socket:WebSocket,state:Arc<SharedState>,code:String) {
    println!("new handle socket");
    let sender = Arc::clone(&state.sender.lock().await.get(&code).unwrap());
    while let Some(o) = socket.recv().await {
        if let Ok(o) = o {
            match o {
                Message::Text(_) => {
                },
                Message::Binary(b) => {
                    let res = sender.try_send(b);
                    if  res.is_err() {
                        let error = format!("{:?}",res);
                        println!("Error sending data {error}");
                        if error.contains("full") { continue; } else { break; } }
                },
                Message::Close(_) => {
                    return;
                },
                _ => {}
            }
        } else {
            println!("{o:?}");
            return;
        }
    }
}
