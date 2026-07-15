use crate::types::{Order, OrderId, Price, Quantity, Side};
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
