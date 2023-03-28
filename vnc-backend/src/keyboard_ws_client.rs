use async_mutex::Mutex;
use crate::SharedState;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade},Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn keyboard_ws_client_handler(
    ws:WebSocketUpgrade,
    State(state):State<Arc<SharedState>>,
    Path(code): Path<String>
    ) -> impl IntoResponse {
    if !state.keyboard_receiver.lock().await.contains_key(&code) {
        println!("insert keyboard ws client");
        let (sender,receiver) = async_channel::bounded(5);
        state.keyboard_receiver.lock().await.insert(code.clone(),Arc::new(Mutex::new(receiver)));
        state.keyboard_sender.lock().await.insert(code.clone(),Arc::new(sender));
    }
    println!("connecting");
    ws.on_upgrade(move |socket| keyboard_ws_client(socket,state,code))
}
pub async fn keyboard_ws_client(
        mut socket:WebSocket,
        state:Arc<SharedState>,
        code:String,
        ) {
    let receiver = Arc::clone(&state.keyboard_receiver.lock().await.get(&code).unwrap());
    println!("upgraded");
    loop {
        if let Ok(movement) = receiver.lock().await.recv().await {
           println!("got data: {}",&movement);
           let res = socket.send(Message::Text(movement)).await;
           if res.is_err() {
                println!("error in function keyboard socket {:?}",res);
           } else {
               println!("send to user");
           }
        } else {
            println!("error occurred obtaining data")
        }
    }
}


