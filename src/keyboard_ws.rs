use async_mutex::Mutex;
use crate::SharedState;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade},Path, State},
    response::IntoResponse,
};
use std::sync::Arc;
pub async fn keyboard_websocket_handler (
    ws:WebSocketUpgrade,
    State(state):State<Arc<SharedState>>,
    Path(code): Path<String>
    ) -> impl IntoResponse {
    if !state.keyboard_receiver.lock().await.contains_key(&code) {
        let (sender,receiver) = async_channel::unbounded();
        state.keyboard_receiver.lock().await.insert(code.clone(), Arc::new(Mutex::new(receiver)));
        state.keyboard_sender.lock().await.insert(code.clone(), Arc::new(sender));
    }
    ws.on_upgrade(move |socket| keyboard_websocket(socket,state,code))
}
pub async fn keyboard_websocket(mut socket:WebSocket,state:Arc<SharedState>,code:String) {
    let sender = state.keyboard_sender.lock().await;
    let sender = Arc::clone(&sender.get(&code).unwrap());
    while let Some(o) = socket.recv().await {
        if let Ok(o) = o {
            match o {
                Message::Text(text) => {
                        println!("{}",&text);
                    if sender.send(text).await.is_err() {
                        println!("success");
                    } else {
                        println!("sent command");
                    }
                },
                _ => {}
            }
        } else {
            return;
        }
    }
}


