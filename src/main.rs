extern crate matched_trading;
extern crate jsonrpc_http_server;
extern crate serde;
#[macro_use] extern crate serde_derive;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use matched_trading::{OrderBook, Direction};
use jsonrpc_http_server::*;
use jsonrpc_http_server::jsonrpc_core::*;


type AMOrderBook = Arc<Mutex<OrderBook>>;

#[derive(Debug)]
struct OrderBookMap {
    map: BTreeMap<String, AMOrderBook>,
}

impl OrderBookMap {
    fn new() -> Self {
        OrderBookMap {
            map: BTreeMap::new(),
        }
    }

    fn init(&mut self, keys: Vec<String>) {
        for k in keys.iter() {
            let order_book = Arc::new(Mutex::new(OrderBook::new(8, 8)));
            self.insert(k.clone(), order_book);
        }
    }

    fn insert(&mut self, k: String, v: AMOrderBook) -> Option<AMOrderBook> {
        self.map.insert(k, v)
    }

    fn remove(&mut self, k: &String) -> Option<AMOrderBook> {
        self.map.remove(k)
    }

    fn get(&self, k: &String) -> Option<&AMOrderBook> {
        self.map.get(k)
    }
}

fn main() {
    let pairs = vec![
        String::from("cet_eos"),
        String::from("otc_eos"),
        String::from("iq_eos"),
        String::from("pub_eos"),
    ];

    let order_book_map = Arc::new(Mutex::new(OrderBookMap::new()));

    order_book_map.lock().unwrap().init(pairs);

    let mut io = IoHandler::default();

    {
        let order_book_map = order_book_map.clone();

        #[derive(Deserialize, Debug)]
        struct CreateOrderBookParams {
            code: String,
        }

        io.add_method("create_order_book", move |params: Params| {
            let p: CreateOrderBookParams = params.parse().unwrap();
            let order_book = Arc::new(Mutex::new(OrderBook::new(8, 8)));

            match order_book_map.lock().unwrap().insert(p.code, order_book) {
                Some(_) => Ok(Value::String("success".into())),
                None => Ok(Value::String("failed".into())),
            }
        });
    }

    {
        let order_book_map = order_book_map.clone();

        #[derive(Deserialize, Debug)]
        struct RemoveOrderBookParams {
            code: String,
        }

        io.add_method("remove_order_book", move |params: Params| {
            let p: RemoveOrderBookParams = params.parse().unwrap();

            match order_book_map.lock().unwrap().remove(&p.code) {
                Some(_) => Ok(Value::String("success".into())),
                None => Ok(Value::String("failed".into())),
            }
        });
    }

    {
        let order_book_map = order_book_map.clone();

        io.add_method("list", move |_| {
            let order_book_map = order_book_map.lock().unwrap();
            let keys: Vec<_> = order_book_map.map.keys().cloned().collect();
            let serialized = serde_json::to_string(&keys).unwrap();

            Ok(serde_json::from_str(&serialized).unwrap())
        });
    }

    {
        let order_book_map = order_book_map.clone();

        #[derive(Deserialize, Debug)]
        struct OrderBookParams {
            code: String,
        }

        io.add_method("order_book", move |params: Params| {
            let p: OrderBookParams = params.parse().unwrap();
            let order_book_map = order_book_map.lock().unwrap();
            let order_book = order_book_map.get(&p.code).unwrap().lock().unwrap();
            let serialized = serde_json::to_string(&*order_book).unwrap();

            Ok(serde_json::from_str(&serialized).unwrap())
        });
    }

    {
        let order_book_map = order_book_map.clone();

        #[derive(Deserialize, Debug)]
        struct SubmitOrderParams {
            code: String,
            direction: Direction,
            id: usize,
            price: f64,
            volume: f64,
        }

        io.add_method("submit_order", move |params: Params| {
            let p: SubmitOrderParams = params.parse().unwrap();
            let order_book_map = order_book_map.lock().unwrap();
            let mut order_book = order_book_map.get(&p.code).unwrap().lock().unwrap();

            order_book.add(p.direction, p.id, p.price, p.volume);

            let resp = order_book.trade();
            let serialized = serde_json::to_string(&*resp).unwrap();

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