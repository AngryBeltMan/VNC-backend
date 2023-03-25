use async_mutex::Mutex;
use crate::SharedState;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade},Path, State},
    response::IntoResponse,
};
use std::sync::Arc;
pub async fn frames_ws_handler(
    ws:WebSocketUpgrade,
    State(state):State<Arc<SharedState>>,
    Path(code): Path<String>
    ) -> impl IntoResponse{
    if !state.sender.lock().await.contains_key(&code) {
        let (sender,reciever) = async_channel::unbounded();
        state.sender.lock().await.insert(code.clone(), sender.into());
        state.receiver.lock().await.insert(code.clone(), Arc::new(Mutex::new(reciever)));
        println!("new ws handler inserted");
    }
    ws.on_failed_upgrade(|_| println!("failed to upgrade"))
        .on_upgrade(move |socket| frames_socket(socket,state,code))
}

pub async fn frames_socket(
    mut socket:WebSocket,
    state:Arc<SharedState>,
    code:String
    ) {
    let receiver = Arc::clone(&state.receiver.lock().await.get(&code).unwrap());
    // while let Ok(frames) = receiver.lock().await.recv().await  {
    loop {
        if let Ok(frames) = receiver.lock().await.recv().await {
           let res = socket.send(Message::Binary(frames)).await;
           if  res.is_err() {
               let error = format!("{:?}",res);
               println!("error in frames socket {error}");
               if error.contains("pipe") {
                   return;
               }
           } else {
               println!("sent");
           }
        }
    }
}
