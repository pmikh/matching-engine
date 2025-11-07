use crate::matching::state::AppState;
use actix_web::{Error, HttpRequest, HttpResponse, web};
use actix_ws::Message;
use log::info;
use tokio_stream::StreamExt;

pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (res, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    let mut ws_rx = data.ws_tx.subscribe();

    info!("WebSocket connection established");

    actix_web::rt::spawn(async move {
        loop {
            tokio::select! {
                Ok(update) = ws_rx.recv() => {
                    if let Ok(json) = serde_json::to_string(&update) {
                        if session.text(json).await.is_err() {
                            break;
                        }
                    }
                }
                Some(Ok(msg)) = msg_stream.next() => {
                    if matches!(msg, Message::Close(_)) {
                        break;
                    }
                }
                else => break,
            }
        }
        info!("WebSocket connection closed");
    });

    Ok(res)
}
