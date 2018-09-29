/// # 初始化订单池 (保留的小数点位数)
/// let mut order_book = OrderBook::new(6, 6);
/// # 插入卖单 (买卖方向，订单id，价格，数量)
/// order_book.add(Direction::Ask, 1, 0.666, 1000.0);
/// # 插入买单 (买卖方向，订单id，价格，数量)
/// order_book.add(Direction::Bid, 2, 0.6660, 666.0);
/// order_book.add(Direction::Bid, 3, 0.6660, 777.0);
///
/// # 撮合, 会返回撮合好的订单状态 (ID，未成交量)
/// order_book.trade()
///
extern crate rust_decimal;
extern crate num_traits;

#[macro_use]
extern crate serde_derive;
extern crate serde;

use std::collections::{BTreeMap, BinaryHeap, VecDeque};
use std::cmp::Ordering;
use std::ops::{AddAssign, SubAssign};

use rust_decimal::Decimal;
use num_traits::cast::FromPrimitive;
use num_traits::ToPrimitive;
use serde::ser::{Serialize, SerializeStruct, Serializer};

#[allow(dead_code)]
enum Rounding {
    Round,
    Ceiling,
    Floor,
}

fn decimal_round(d: Decimal, places: i32, mode: Rounding) -> Decimal {
    let n = Decimal::from_f64(10.0f64.powi(places)).unwrap();

    let d = d * n;

    let r = match mode {
        Rounding::Round => d.round(),
        Rounding::Ceiling => d.ceil(),
        Rounding::Floor => d.floor(),
    };

    r / n
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum Direction {
    Ask,
    Bid,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Price {
    direction: Direction,
    value: Decimal,
}

impl Price {
    fn new(direction: Direction, n: f64) -> Self {
        Price {
            direction,
            value: Decimal::from_f64(n).unwrap(),
        }
    }

    fn floor(&mut self, places: i32) {
        self.value = decimal_round(self.value, places, Rounding::Floor);
    }
}

impl Serialize for Price {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&*format!("{:.8}", self.value.to_f64().unwrap()))
    }
}

impl Ord for Price {
    fn cmp(&self, other: &Price) -> Ordering {
        use Direction::*;

        match (&self.direction, &other.direction) {
            (&Ask, &Ask) => other.value.cmp(&self.value),
            (&Bid, &Bid) => self.value.cmp(&other.value),
            (_, _) => Ordering::Equal,
        }
    }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Price) -> Option<Ordering> {
        use Direction::*;

        match (&self.direction, &other.direction) {
            (&Ask, &Ask) => Some(other.value.cmp(&self.value)),
            (&Bid, &Bid) => Some(self.value.cmp(&other.value)),
            (_, _) => None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Volume {
    value: Decimal,
}

impl Volume {
    fn new(n: f64) -> Self {
        Volume {
            value: Decimal::from_f64(n).unwrap(),
        }
    }

    fn floor(&mut self, places: i32) {
        self.value = decimal_round(self.value, places, Rounding::Floor);
    }
}

impl Serialize for Volume {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&*format!("{:.8}", self.value.to_f64().unwrap()))
    }
}

impl AddAssign for Volume {
    fn add_assign(&mut self, other: Volume) {
        *self = Volume {
            value: self.value + other.value,
        }
    }
}

impl SubAssign for Volume {
    fn sub_assign(&mut self, other: Volume) {
        *self = Volume {
            value: self.value - other.value,
        }
    }
}

#[derive(Serialize, Debug)]
struct VolumeCollection {
    total: Volume,
    deque: VecDeque<(usize, Volume)>
}

impl VolumeCollection {
    fn new() -> Self {
        VolumeCollection {
            total: Volume::new(0.0),
            deque: VecDeque::new(),
        }
    }

    fn push(&mut self, id: usize, volume: Volume) {
        self.deque.push_back((id, volume.clone()));
        self.total += volume;
    }

    fn is_empty(&self) -> bool {
        self.deque.is_empty()
    }

    fn deque_pop_front(&mut self) {
        if let Some((_, volume)) = self.deque.pop_front() {
            self.total -= volume;
        }
    }
}

#[derive(Serialize, Debug)]
struct BookMap {
    map: BTreeMap<Price, VolumeCollection>,
}

impl BookMap {
    fn new() -> Self {
        BookMap {
            map: BTreeMap::new(),
        }
    }

    fn remove_deque(&mut self, price: &Price) {
        if let Some(vc) = self.map.get_mut(price) {
            vc.deque_pop_front();
        }
    }

    fn remove_key(&mut self, price: &Price) {
        if self.deque_is_empty(price) {
            self.map.remove(price);
        }
    }

    fn update_key(&mut self, price: &Price, d: Decimal) {
        if let Some(vc) = self.map.get_mut(price) {
            if let Some((_, volume)) = vc.deque.front_mut() {
                vc.total -= Volume { value: volume.value - d };
                *volume = Volume { value: d };
            }
        }
    }

    fn deque_is_empty(&self, price: &Price) -> bool {
        match self.map.get(price) {
            Some(vc) => vc.is_empty(),
            None => false,
        }
    }

    fn first_order(&self, price: &Price) -> Option<(usize, Volume)> {
        if let Some(vc) = self.map.get(&price) {
            if let Some((id, volume)) = vc.deque.front() {
                return Some((*id, volume.clone()));
            }
        }

        None
    }
}

#[derive(Serialize, Debug)]
struct Book {
    kv: BookMap,
    heap: BinaryHeap<Price>
}

impl Book {
    fn new() -> Self {
        Book {
            kv: BookMap::new(),
            heap: BinaryHeap::new(),
        }
    }

    fn insert(&mut self, id: usize, price: Price, volume: Volume) {
        if !self.kv.map.contains_key(&price) {
            self.heap.push(price.clone());
        }

        let vc = self.kv.map.entry(price.clone()).or_insert(VolumeCollection::new());
        vc.push(id, volume.clone());
    }

    fn first_order(&self) -> Option<(usize, Volume)> {
        match self.first_price() {
            Some(price) => self.kv.first_order(&price),
            None => None,
        }
    }

    fn first_price(&self) -> Option<Price> {
        match self.heap.peek() {
            Some(price) => Some(price.clone()),
            None => None,
        }
    }

    fn remove_first(&mut self) {
        if let Some(price) = self.first_price() {
            self.kv.remove_deque(&price);

            if self.kv.deque_is_empty(&price) {
                self.kv.remove_key(&price);
                self.heap.pop();
            }
        }
    }

    fn update_first(&mut self, d: Decimal) {
        match d.cmp(&Decimal::from_f64(0.0).unwrap()) {
            Ordering::Equal => {
                self.remove_first();
            },
            _ => {
                if let Some(price) = self.first_price() {
                    self.kv.update_key(&price, d);
                }
            }
        }
    }
}

#[derive(Serialize, Debug, Clone)]
struct Fixed {
    base: i32,
    quote: i32,
}

impl Fixed {
    fn new(base: i32, quote: i32) -> Self {
        Fixed {
            base,
            quote,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct OrderBook {
    fixed: Fixed,
    ask: Book,
    bid: Book,
}

impl OrderBook {
    pub fn new(base_fixed: i32, quote_fixed: i32) -> Self {
        let fixed = Fixed::new(base_fixed, quote_fixed);

        OrderBook {
            fixed,
            ask: Book::new(),
            bid: Book::new(),
        }
    }

    pub fn add(&mut self, direction: Direction, id: usize, price: f64, volume: f64) {
        use Direction::*;

        let mut price = Price::new(direction.clone(), price);
        price.floor(self.fixed.quote);

        let mut volume = Volume::new(volume);
        volume.floor(self.fixed.base);

        match direction {
            Ask => self.ask.insert(id, price, volume),
            Bid => self.bid.insert(id, price, volume),
        };
    }

    pub fn trade(&mut self) -> Vec<(usize, Volume)> {
        let mut result = Vec::new();
        let z = Decimal::from_f64(0.0).unwrap();

        while self.is_matching() {
            match (self.ask.first_order(), self.bid.first_order()) {
                (Some((a_id, a_volume)), Some((b_id, b_volume))) => {
                    let r = a_volume.value - b_volume.value;

                    match r.cmp(&z) {
                        Ordering::Equal => {
                            result.push((a_id, Volume { value: z }));
                            result.push((b_id, Volume { value: z }));

                            self.ask.remove_first();
                            self.bid.remove_first();
                        },
                        Ordering::Less => {
                            result.push((a_id, Volume { value: z }));
                            result.push((b_id, Volume { value: -r }));

                            self.ask.remove_first();
                            self.bid.update_first(-r);
                        },
                        Ordering::Greater => {
                            result.push((a_id, Volume { value: r }));
                            result.push((b_id, Volume { value: z }));

                            self.ask.update_first(r);
                            self.bid.remove_first();
                        },
                    };
                },
                (_, _) => (),
            }
        }

        result
    }

    fn is_matching(&self) -> bool {
        match (self.bid.first_price(), self.ask.first_price()) {
            (Some(bid_price), Some(ask_price)) => {
                bid_price.value >= ask_price.value
            },
            (_, _) => false,
        }
    }
}