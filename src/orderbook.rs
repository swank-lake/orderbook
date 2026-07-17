use crate::types::{Order, OrderId, Price, Quantity, Side, Trade};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, VecDeque};

#[derive(Debug, PartialEq)]
pub struct PriceLevel {
    price: Price,
    orders: VecDeque<Order>,
    total_quantity: Quantity,
}

impl PriceLevel {
    //create new price level
    pub fn new(price: Price) -> PriceLevel {
        PriceLevel {
            price,
            orders: VecDeque::new(),
            total_quantity: Quantity::new(0),
        }
    }

    //add order to a price level queue  it belongs
    pub fn add_order_to_price_level(&mut self, order: Order) {
        self.total_quantity =
            Quantity::new(self.total_quantity.get_quantity() + order.get_quantity().get_quantity());

        //push order to back of price level queue
        self.orders.push_back(order);
    }

    //Check if queue is empty
    pub fn is_queue_empty(&self) -> bool {
        return self.orders.is_empty();
    }

    //walk the queue
    // match against a resting order and return a trade
    pub fn match_within_level(&mut self, order: &mut Order) -> (Vec<Trade>, Vec<OrderId>) {
        let mut trades = Vec::new();
        let mut removed_order_ids = Vec::new();

        //loop runs as long as the order has quantity and the queue isn't empty
        while order.get_quantity().get_quantity() > 0 && !self.is_queue_empty() {
            let resting_order = self
                .orders
                .front_mut()
                .expect("queue should not be empty here");

            let incoming_quantity = order.get_quantity().get_quantity();
            let resting_quantity = resting_order.get_quantity().get_quantity();

            match incoming_quantity.cmp(&resting_quantity) {
                Ordering::Less => {
                    //resting order absorbs the incoming order and still has quantity leftover
                    // it stays at the front of the queue with a reduced quantity
                    resting_order.set_quantity(Quantity::new(resting_quantity - incoming_quantity));

                    //incoming order is fully satsified so its remaining quantity goes to zero
                    order.set_quantity(Quantity::new(0));

                    //build the trade,it executes at the resting order's price
                    let trade = Trade::new(
                        resting_order.get_price(),
                        Quantity::new(incoming_quantity),
                        order.get_id(),
                        resting_order.get_id(),
                    );

                    //store the trade
                    trades.push(trade);
                }
                Ordering::Greater => {
                    //grab what is needed from resting order before popping it
                    let resting_id = resting_order.get_id();
                    let resting_price = resting_order.get_price();

                    //resting order is fully consumed, so remove it from the queue completely
                    self.orders.pop_front();

                    //keep total quantity in sync now that this order has left the level
                    self.total_quantity =
                        Quantity::new(self.total_quantity.get_quantity() - resting_quantity);

                    //reduce incoming order quantity by how much quantity the resting order consumed
                    order.set_quantity(Quantity::new(incoming_quantity - resting_quantity));

                    //build the trade, executes at resting order's price
                    let trade = Trade::new(
                        resting_price,
                        Quantity::new(resting_quantity),
                        order.get_id(),
                        resting_id,
                    );

                    trades.push(trade);
                    removed_order_ids.push(resting_id);
                }

                Ordering::Equal => {
                    //grab what is needed from resting order before popping it
                    let resting_id = resting_order.get_id();
                    let resting_price = resting_order.get_price();

                    //resting order is fully consumed, remove it from the queue completely
                    self.orders.pop_front();

                    //update total quantity
                    self.total_quantity =
                        Quantity::new(self.total_quantity.get_quantity() - resting_quantity);

                    //set incoming order quantity to zero
                    order.set_quantity(Quantity::new(0));

                    //build the trade, executes at resting order's price
                    let trade = Trade::new(
                        resting_price,
                        Quantity::new(resting_quantity),
                        order.get_id(),
                        resting_id,
                    );

                    trades.push(trade);
                }
            }
        }
        (trades, removed_order_ids)
    }
}

#[derive(Debug, PartialEq)]
pub struct OrderLocation {
    side: Side,
    price: Price,
}

#[derive(Debug, PartialEq)]
pub struct OrderBook {
    bid: BTreeMap<Price, PriceLevel>,
    ask: BTreeMap<Price, PriceLevel>,
    order_search: HashMap<OrderId, OrderLocation>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            bid: BTreeMap::new(),
            ask: BTreeMap::new(),
            order_search: HashMap::new(),
        }
    }

    pub fn get_best_bid(&self) -> Option<Price> {
        return self.bid.keys().next_back().copied();
    }

    pub fn get_best_ask(&self) -> Option<Price> {
        return self.ask.keys().next().copied();
    }

    //find the price level on the opposite side of the book
    pub fn get_best_opposite_level(&mut self, side: Side) -> Option<&mut PriceLevel> {
        match side {
            Side::Buy => {
                let best_ask_price = self.get_best_ask()?;
                self.ask.get_mut(&best_ask_price)
            }
            Side::Sell => {
                let best_bid_price = self.get_best_bid()?;
                self.ask.get_mut(&best_bid_price)
            }
        }
    }

    pub fn process_order(&mut self, order: Order) -> (Vec<Trade>, Quantity) {
        //make the order mutable in order to shrink its quantity as matching proceeds
        let mut remaining_order = order;

        //collect every trade produced
        let mut trades = Vec::new();

        //match the orders as long as incoming order still has quantity
        // and the top of the opposite side of the book still crosses it
        while remaining_order.get_quantity().get_quantity() > 0
            && self.check_if_order_crosses(remaining_order.get_price(), remaining_order.get_side())
        {
            //find best priced level on opposite side
            let price_level = self
                .get_best_opposite_level(remaining_order.get_side())
                .expect("crosses check guarantees a level exists");

            //consume the price level's queue as much as possible
            let (mut level_trades, removed_order_ids) =
                price_level.match_within_level(&mut remaining_order);

            let level_is_empty = price_level.is_queue_empty();

            //remove every fully consumed resting order from order search too
            for id in removed_order_ids {
                self.order_search.remove(&id);
            }

            trades.append(&mut level_trades);

            //if that price level's queue is fully drained, delete it
            if level_is_empty {
                match remaining_order.get_side() {
                    Side::Buy => {
                        let best_ask = self
                            .get_best_ask()
                            .expect("price exists because level exists");
                        self.ask.remove(&best_ask);
                    }
                    Side::Sell => {
                        let best_bid = self
                            .get_best_bid()
                            .expect("price exists because level exists");
                        self.bid.remove(&best_bid);
                    }
                }
            }
        }
        let leftover = remaining_order.get_quantity();
        (trades, leftover)
    }

    //check if an incoming order crosses the spread
    pub fn check_if_order_crosses(&self, price: Price, side: Side) -> bool {
        //check side of the order
        match side {
            Side::Buy => {
                //if there's a best ask, check if price crosses it
                // if there's no best ask at all return false
                self.get_best_ask()
                    .map_or(false, |best_ask| price >= best_ask)
            }

            Side::Sell => {
                //if there's a best bid, check if price crosses it
                // if there's no best bid at all, return false
                self.get_best_bid()
                    .map_or(false, |best_bid| price <= best_bid)
            }
        }
    }

    pub fn insert_new_order(&mut self, order: Order) {
        //find whether order is bid/ask
        let book_side = match order.get_side() {
            Side::Buy => &mut self.bid,
            Side::Sell => &mut self.ask,
        };

        //find if price level exists
        // if it doesn't, build a fresh price level
        // store the mutable reference in a variable
        let price_level = book_side
            .entry(order.get_price())
            .or_insert_with(|| PriceLevel::new(order.get_price()));

        self.order_search.insert(
            order.get_id(),
            OrderLocation {
                side: order.get_side(),
                price: order.get_price(),
            },
        );

        price_level.add_order_to_price_level(order);
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> bool {
        //check if order exists
        match self.order_search.get(&order_id) {
            //if order exists
            Some(location) => {
                //store the side where the order lives
                let book_side = match location.side {
                    Side::Buy => &mut self.bid,
                    Side::Sell => &mut self.ask,
                };

                //find the price level where the order resides

                let price_level = book_side
                    .get_mut(&location.price)
                    .expect("order_search points to a price level that doesn't exist");

                //search for the order id in the queue
                let index = price_level
                    .orders
                    .iter()
                    .position(|o| o.get_id() == order_id)
                    .expect("order does not exist at this price level");

                //delete the order
                let removed_order = price_level
                    .orders
                    .remove(index)
                    .expect("index out of bounds");

                //update the total quantity of that price level
                price_level.total_quantity = Quantity::new(
                    price_level.total_quantity.get_quantity()
                        - removed_order.get_quantity().get_quantity(),
                );

                //check if the price level is empty
                let should_remove_level = price_level.is_queue_empty();

                //delete price level if empty
                if should_remove_level {
                    book_side.remove(&location.price);
                }

                //delete the order from the hash map
                self.order_search.remove(&order_id);

                true
            }
            None => false,
        }
    }
}
