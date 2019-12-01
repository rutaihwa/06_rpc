extern crate jsonrpc as jsonrpc_client;

use failure::Error;
use jsonrpc_client::{client::Client, error::Error as ClientError};
use jsonrpc_http_server::jsonrpc_core::{Error as ServerError, IoHandler, Value};
use jsonrpc_http_server::ServerBuilder;
use log::{debug, error, trace};
use serde::Deserialize;
use std::{
    env, fmt,
    net::SocketAddr,
    sync::{
        mpsc::{channel, Sender},
        Mutex,
    },
    thread,
};

const START_ROLL_CALL: &str = "start_roll_call";
const MARK_ITSELF: &str = "mark_itself";

struct Remote {
    client: Client,
}

// Calling remote services
impl Remote {
    // New remote client
    fn new(addr: SocketAddr) -> Self {
        let url = format!("http://{}", addr);
        let client = Client::new(url, None, None);
        Self { client }
    }

    // Helping to call remote methods
    fn call_method<T>(&self, method: &str, args: &[Value]) -> Result<T, ClientError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let request = self.client.build_request(method, args);
        self.client
            .send_request(&request)
            .and_then(|res| res.into_result::<T>())
    }

    fn start_roll_call(&self) -> Result<bool, ClientError> {
        self.call_method(START_ROLL_CALL, &[])
    }

    fn mark_itself(&self) -> Result<bool, ClientError> {
        self.call_method(MARK_ITSELF, &[])
    }
}

// Action struct a worker
enum Action {
    StartRollCall,
    MarkItSelf,
}

// Spawn a worker
fn spawn_worker() -> Result<Sender<Action>, Error> {
    let (tx, rx) = channel();
    let next: SocketAddr = env::var("NEXT")?.parse()?;
    thread::spawn(move || {
        let remote = Remote::new(next);
        let mut in_roll_call = false;
        for action in rx.iter() {
            match action {
                Action::StartRollCall => {
                    if !in_roll_call {
                        if remote.start_roll_call().is_ok() {
                            debug!("ON");
                            in_roll_call = true;
                        }
                    } else {
                        if remote.mark_itself().is_ok() {
                            debug!("OFF");
                            in_roll_call = false;
                        }
                    }
                }
                Action::MarkItSelf => {
                    if in_roll_call {
                        debug!("OFF");
                        in_roll_call = false;
                    } else {
                        debug!("SKIP");
                    }
                }
            }
        }
    });
    Ok(tx)
}

// Dealing with errors
fn to_internal<E: fmt::Display>(err: E) -> ServerError {
    error!("Error: {}", err);
    ServerError::internal_error()
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let tx = spawn_worker()?;
    let addr: SocketAddr = env::var("ADDRESS")?.parse()?;
    let mut io = IoHandler::default();
    let sender = Mutex::new(tx.clone());

    io.add_method(START_ROLL_CALL, move |_| {
        trace!("START_ROLL_CALL");

        let tx = sender.lock().map_err(to_internal)?;

        tx.send(Action::StartRollCall)
            .map_err(to_internal)
            .map(|_| Value::Bool(true))
    });

    let sender = Mutex::new(tx.clone());
    io.add_method(MARK_ITSELF, move |_| {
        trace!("MARK_ITSELF");
        let tx = sender.lock().map_err(to_internal)?;
        tx.send(Action::MarkItSelf)
            .map_err(to_internal)
            .map(|_| Value::Bool(true))
    });

    let server = ServerBuilder::new(io).threads(3).start_http(&addr)?;

    Ok(server.wait())
}
