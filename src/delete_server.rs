use axum::extract::{Path, State};
use std::sync::Arc;
use crate::SharedState;
pub async fn delete_server(Path(code):Path<String>,State(state): State<Arc<SharedState>>) {
    state.sender.lock().await.remove(&code).unwrap();
    state.keyboard_sender.lock().await.remove(&code).unwrap();
    state.receiver.lock().await.remove(&code).unwrap();
    state.keyboard_receiver.lock().await.remove(&code).unwrap();
}
