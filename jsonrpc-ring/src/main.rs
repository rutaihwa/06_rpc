use failure::Error;
use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;
use log::{debug, error, trace};
use std::{env, net::SocketAddr};

const START_ROLL_CALL: &str = "start_roll_call";
const MARK_ITSELF: &str = "mark_itself";

fn main() -> Result<(), Error> {
    env_logger::init();

    let addr: SocketAddr = env::var("ADDRESS")?.parse()?;

    let mut io = IoHandler::new();
    io.add_method("say_hello", |_params: Params| {
        Ok(Value::String("hello".to_string()))
    });

    let server = ServerBuilder::new(io).threads(3).start_http(&addr)?;

    Ok(server.wait())
}
