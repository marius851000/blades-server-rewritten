use std::time::Duration;

use actix_web::{
    HttpRequest, HttpResponse, get,
    http::header::{HeaderName, HeaderValue},
    rt, web,
};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;
use tokio::{select, sync::mpsc, time::sleep};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{BladeApiError, arena::MatchmakingMessage, session::SessionLookedUpMaybe};

#[get("/blades.bgs.services/api/rms/v1/public/")]
async fn matchmaking_ws(
    req: HttpRequest,
    stream: web::Payload,
    user_session: SessionLookedUpMaybe,
) -> Result<HttpResponse, BladeApiError> {
    let user_session = user_session.get_session_or_error()?;

    let (mut res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    //TODO: verify it gets auto-disconnected if client is lost
    let (tx, rx) = mpsc::unbounded_channel::<MatchmakingMessage>();
    let mut rx = UnboundedReceiverStream::new(rx);

    {
        let mut matchmaking_ws = user_session.session.matchmaking_ws.lock().await;
        *matchmaking_ws = Some(tx);
    }

    let user_session_cloned = user_session.session.clone();
    rt::spawn(async move {
        // spawn another thread to catch panic
        let thread = rt::spawn(async move {
            loop {
                select! {
                    Some(msg) = stream.next() => {
                        match msg {
                            Ok(AggregatedMessage::Text(_text)) => {
                                todo!("handle text message");
                            }

                            Ok(AggregatedMessage::Binary(_bin)) => {
                                todo!("handle binary message");
                            }

                            Ok(AggregatedMessage::Ping(msg)) => {
                                // respond to PING frame with PONG frame
                                session.pong(&msg).await.unwrap();
                            }

                            _ => {}
                        }
                    }
                    _ = sleep(Duration::from_secs(10)) => {
                        session.ping(b"").await.unwrap();
                    }
                    Some(msg) = rx.next() => {
                        match msg {

                        }
                    }
                    else => {
                        break;
                    }
                }
            }
        });

        match thread.await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Caught error in websocket thread: {}", e)
            }
        };

        let mut matchmaking_ws = user_session_cloned.matchmaking_ws.lock().await;
        *matchmaking_ws = None;
    });

    // respond immediately with response connected to WS session

    res.headers_mut().append(
        HeaderName::from_static("sec-websocket-protocol"),
        HeaderValue::from_static("json"),
    );
    Ok(res)
}
