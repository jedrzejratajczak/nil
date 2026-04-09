use std::os::unix::net::UnixStream;
use std::time::Duration;

use greetd_ipc::codec::SyncCodec;
use greetd_ipc::{AuthMessageType, Request, Response};
use zeroize::Zeroizing;

const SESSION_CMD: &[&str] = &["uwsm", "start", "hyprland-uwsm.desktop"];
const SOCKET_TIMEOUT: Duration = Duration::from_secs(4);

pub fn authenticate(username: &str, password: &Zeroizing<String>) -> Result<(), String> {
    let sock_path = std::env::var("GREETD_SOCK").map_err(|_| "GREETD_SOCK not set".to_string())?;

    let mut stream = UnixStream::connect(&sock_path).map_err(|e| format!("Connect failed: {e}"))?;

    stream.set_read_timeout(Some(SOCKET_TIMEOUT)).ok();
    stream.set_write_timeout(Some(SOCKET_TIMEOUT)).ok();

    Request::CreateSession {
        username: username.to_string(),
    }
    .write_to(&mut stream)
    .map_err(|e| format!("CreateSession failed: {e}"))?;

    let result = (|| {
        loop {
            match Response::read_from(&mut stream).map_err(|e| format!("Read failed: {e}"))? {
                Response::AuthMessage {
                    auth_message_type, ..
                } => {
                    let response = match auth_message_type {
                        AuthMessageType::Secret | AuthMessageType::Visible => {
                            Some(password.to_string())
                        }
                        AuthMessageType::Info | AuthMessageType::Error => None,
                    };

                    Request::PostAuthMessageResponse { response }
                        .write_to(&mut stream)
                        .map_err(|e| format!("PostAuth failed: {e}"))?;
                }
                Response::Success => break,
                Response::Error { description, .. } => return Err(description),
            }
        }

        Request::StartSession {
            cmd: SESSION_CMD.iter().map(|s| s.to_string()).collect(),
            env: vec![],
        }
        .write_to(&mut stream)
        .map_err(|e| format!("StartSession failed: {e}"))?;

        match Response::read_from(&mut stream) {
            Ok(Response::Success) => Ok(()),
            Ok(Response::Error { description, .. }) => Err(description),
            Ok(_) => Err("Unexpected response".to_string()),
            Err(e) => Err(format!("Read failed: {e}")),
        }
    })();

    if result.is_err() {
        if let Err(e) = Request::CancelSession.write_to(&mut stream) {
            eprintln!("Warning: failed to cancel session: {e}");
        }
    }

    result
}
