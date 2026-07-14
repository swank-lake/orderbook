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

    //add order to a price level it belongs
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
