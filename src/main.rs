use std::{net::SocketAddr, fmt::format};
use async_channel;
use tokio::sync::mpsc;
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
struct SharedState {
    sender:async_channel::Sender<Vec<u8>>,
    receiver:Mutex<async_channel::Receiver<Vec<u8>>>,
    keyboard_sender:async_channel::Sender<String>,
    keyboard_receiver:Mutex<async_channel::Receiver<String>>,
}
#[tokio::main]
async fn main() {
    let (sender,receiver) = async_channel::bounded(1);
    let (keyboard_sender,keyboard_receiver) = async_channel::unbounded();
    let app_state = Arc::new(SharedState {
        sender,
        keyboard_sender,
        receiver:Mutex::new(receiver),
        keyboard_receiver:Mutex::new(keyboard_receiver),
    });

    let app = Router::new()
        .route("/ws",get(ws_handler))
        .route("/ws/frames",get(frames_ws_handler))
        .route("/ws/keyboard",get(keyboard_websocket_handler))
        .route("/ws/client/keyboard",get(keyboard_ws_client_handler))
        .with_state(app_state)
        .layer(
            CorsLayer::new()
            .allow_origin("*".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET,Method::POST])
        );
    let addr = SocketAddr::from(([127,0,0,1],6969));
    println!("listening on address {}",&addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await.unwrap();
}
async fn keyboard_websocket_handler(ws:WebSocketUpgrade,State(state):State<Arc<SharedState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| keyboard_websocket(socket,state))
}
async fn keyboard_websocket(mut socket:WebSocket,state:Arc<SharedState>) {
    while let Some(o) = socket.recv().await {
        if let Ok(o) = o {
            match o {
                Message::Text(text) => {
                    println!("recieved text");
                    if state.keyboard_sender.send(text).await.is_err() {
                        println!("could not send text");
                    } else {
                        println!("sent text");
                    }
                },
                _ => {}
            }
        }
    }
}
async fn keyboard_ws_client_handler(ws:WebSocketUpgrade,State(state):State<Arc<SharedState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| keyboard_ws_client(socket,state))
}
async fn keyboard_ws_client(mut socket:WebSocket,state:Arc<SharedState>) {
    loop {
        if let Ok(movement) = state.keyboard_receiver.lock().await.recv().await {
            println!("client recieved text");
           if socket.send(Message::Text(movement)).await.is_err() {println!("error in function frame socket")}
        } else {
            println!("client could not recieve text");
        }
    }
}
async fn ws_handler(ws:WebSocketUpgrade,State(state):State<Arc<SharedState>>) -> impl IntoResponse{
    ws.on_upgrade(move |socket| handle_socket(socket,state))
}

async fn frames_ws_handler(ws:WebSocketUpgrade,State(state):State<Arc<SharedState>>) -> impl IntoResponse{
    ws.on_failed_upgrade(|_| println!("failed to upgrade")).on_upgrade(move |socket| frames_socket(socket,state))
}

async fn frames_socket(mut socket:WebSocket,state:Arc<SharedState>) {
    if let Ok(frames) = state.receiver.lock().await.recv().await {
       if socket.send(Message::Binary(frames)).await.is_err() {println!("error in function frame socket")}
    }
}
async fn handle_socket(mut socket:WebSocket,state:Arc<SharedState>) {
    while let Some(o) = socket.recv().await {
        println!("recieved data");
        if let Ok(o) = o {
            match o {
                Message::Text(_) => {
                    println!("set text");
                    // *state.frame_buffer.lock().await = Some(text);
                },
                Message::Binary(b) => {
                    println!("set data");
                    if state.sender.try_send(b).is_err() {
                        println!("Error sending data");
                        continue;
                    }
                },
                Message::Ping(_) => {
                    println!("Ping")
                },
                Message::Pong(_) => {
                    println!("Pong")
                },
                Message::Close(_) => {
                    return;
                },
            }
        } else {
            println!("{o:?}");
            return;
        }
    }
}
