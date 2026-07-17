#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Price {
    price_value: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Quantity {
    quantity_value: u64,
}

#[derive(Debug, Clone, Copy, Eq, Ord, Hash, PartialEq, PartialOrd)]
pub struct OrderId {
    order_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, PartialEq)]
pub struct Trade {
    price: Price,
    quantity: Quantity,
    incoming_order_id: OrderId,
    resting_order_id: OrderId,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Order {
    id: OrderId,
    price: Price,
    quantity: Quantity,
    side: Side,
}

impl Order {
    pub fn get_id(&self) -> OrderId {
        return self.id;
    }

    pub fn get_price(&self) -> Price {
        return self.price;
    }

    pub fn get_quantity(&self) -> Quantity {
        return self.quantity;
    }

    pub fn get_side(&self) -> Side {
        return self.side;
    }

    pub fn set_quantity(&mut self, new_quantity: Quantity) {
        self.quantity = new_quantity;
    }

    pub fn new(id: OrderId, price: Price, quantity: Quantity, side: Side) -> Order {
        Order {
            id,
            price,
            quantity,
            side,
        }
    }
}

impl Price {
    //writing these functions inside this block
    // means that they are attached to the struct

    //new price
    pub fn new(value: u64) -> Price {
        Price { price_value: value }
    }

    pub fn get_price(&self) -> u64 {
        return self.price_value;
    }
}

impl Quantity {
    //new quantity
    pub fn new(value: u64) -> Quantity {
        Quantity {
            quantity_value: value,
        }
    }

    pub fn get_quantity(&self) -> u64 {
        return self.quantity_value;
    }
}

impl OrderId {
    //new order id
    pub fn new(value: u64) -> OrderId {
        OrderId { order_id: value }
    }

    pub fn get_order_id(&self) -> u64 {
        return self.order_id;
    }
}

impl Trade {
    pub fn new(
        price: Price,
        quantity: Quantity,
        incoming_order_id: OrderId,
        resting_order_id: OrderId,
    ) -> Trade {
        Trade {
            price,
            quantity,
            incoming_order_id,
            resting_order_id,
        }
    }
}
