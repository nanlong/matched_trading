extern crate matched_trading;
extern crate jsonrpc_http_server;
extern crate serde;
#[macro_use] extern crate serde_derive;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use matched_trading::{OrderBook, Direction};
use jsonrpc_http_server::*;
use jsonrpc_http_server::jsonrpc_core::*;

#[derive(Deserialize, Debug)]
struct ListParams {
    code: String,
}

#[derive(Deserialize, Debug)]
struct SubmitOrderParams {
    code: String,
    direction: Direction,
    id: usize,
    price: f64,
    volume: f64,
}

fn main() {
    let order_book = Arc::new(Mutex::new(OrderBook::new(8, 8)));
    let mut io = IoHandler::default();

    {
        let order_book = order_book.clone();

        io.add_method("list", move |params: Params| {
            let _p: ListParams = params.parse().unwrap();
            let order_book = order_book.lock().unwrap();
            let serialized = serde_json::to_string(&*order_book).unwrap();

            Ok(serde_json::from_str(&serialized).unwrap())
        });
    }

    {
        let order_book = order_book.clone();

        io.add_method("submit_order", move |params: Params| {
            let p: SubmitOrderParams = params.parse().unwrap();
            let mut order_book = order_book.lock().unwrap();

            order_book.add(p.direction, p.id, p.price, p.volume);

            let serialized = serde_json::to_string(&*order_book).unwrap();
            Ok(serde_json::from_str(&serialized).unwrap())
        });
    }

    let server = ServerBuilder::new(io)
        .threads(4)
        .cors(DomainsValidation::AllowOnly(vec![AccessControlAllowOrigin::Null]))
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .expect("Unable to start RPC server");

    server.wait();
}