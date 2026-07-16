use orderbook::orderbook::OrderBook;
use orderbook::types::{Order, OrderId, Price, Quantity, Side};

#[test]
fn test_side_isolation_buy() {
    //create fresh orderbook
    let mut book = OrderBook::new();

    //construct a buy order
    let order = Order::new(OrderId::new(1), Price::new(50), Quantity::new(5), Side::Buy);

    //insert the order in the book
    book.insert_new_order(order);

    //assert best bid returns the inserted price
    assert_eq!(book.get_best_bid(), Some(Price::new(50)));

    //assert best ask returns nothing
    assert_eq!(book.get_best_ask(), None);
}

#[test]
fn test_side_isolation_sell() {
    //create fresh orderbook
    let mut book = OrderBook::new();

    //construct a sell order
    let order = Order::new(
        OrderId::new(1),
        Price::new(45),
        Quantity::new(7),
        Side::Sell,
    );

    //insert order in the book
    book.insert_new_order(order);

    //assert best ask returns the inserted price
    assert_eq!(book.get_best_ask(), Some(Price::new(45)));

    //assert best bid returns nothing
    assert_eq!(book.get_best_bid(), None);
}

#[test]
fn test_best_bid_tracking() {
    //create fresh orderbook
    let mut book = OrderBook::new();

    //construct a buy order
    let order = Order::new(OrderId::new(1), Price::new(50), Quantity::new(5), Side::Buy);

    //insert order in the book
    book.insert_new_order(order);

    //assert best bid is 50
    assert_eq!(book.get_best_bid(), Some(Price::new(50)));

    let order = Order::new(
        OrderId::new(2),
        Price::new(105),
        Quantity::new(10),
        Side::Buy,
    );

    book.insert_new_order(order);

    //assert best bid is 105
    assert_eq!(book.get_best_bid(), Some(Price::new(105)));

    let order = Order::new(OrderId::new(3), Price::new(45), Quantity::new(3), Side::Buy);

    book.insert_new_order(order);

    //assert best bid is still 105
    assert_eq!(book.get_best_bid(), Some(Price::new(105)));
}

#[test]
fn test_best_ask_tracking() {
    //create fresh orderbook
    let mut book = OrderBook::new();

    //construct a sell order
    let order = Order::new(
        OrderId::new(1),
        Price::new(100),
        Quantity::new(5),
        Side::Sell,
    );

    //insert sell order
    book.insert_new_order(order);

    //assert best ask is 100
    assert_eq!(book.get_best_ask(), Some(Price::new(100)));

    let order = Order::new(
        OrderId::new(2),
        Price::new(95),
        Quantity::new(4),
        Side::Sell,
    );

    book.insert_new_order(order);

    //assert best ask is 95
    assert_eq!(book.get_best_ask(), Some(Price::new(95)));

    let order = Order::new(
        OrderId::new(3),
        Price::new(98),
        Quantity::new(10),
        Side::Sell,
    );

    book.insert_new_order(order);

    //assert best ask is still 95
    assert_eq!(book.get_best_ask(), Some(Price::new(95)));
}

#[test]
fn test_order_cancellation() {
    //create empty order book
    let mut book = OrderBook::new();

    //construct first order
    let order = Order::new(
        OrderId::new(1),
        Price::new(105),
        Quantity::new(10),
        Side::Buy,
    );

    //insert first order
    book.insert_new_order(order);

    //construct second order
    let order = Order::new(
        OrderId::new(2),
        Price::new(102),
        Quantity::new(3),
        Side::Buy,
    );

    //insert second order
    book.insert_new_order(order);

    //confirm best bid is 105
    assert_eq!(book.get_best_bid(), Some(Price::new(105)));

    //cancel order at 105
    assert_eq!(book.cancel_order(OrderId::new(1)), true);

    //confirm best bid is 102
    assert_eq!(book.get_best_bid(), Some(Price::new(102)));

    //cancel order at 102
    assert_eq!(book.cancel_order(OrderId::new(2)), true);

    //confirm if all price levels have been removed
    assert_eq!(book.get_best_bid(), None);
}

#[test]
fn bid_ask_independence() {
    //create new orderbook
    let mut book = OrderBook::new();

    //construct a bid order
    let order = Order::new(
        OrderId::new(1),
        Price::new(100),
        Quantity::new(20),
        Side::Buy,
    );

    //insert the bid
    book.insert_new_order(order);

    //construct an ask order
    let order = Order::new(
        OrderId::new(2),
        Price::new(110),
        Quantity::new(5),
        Side::Sell,
    );

    //insert the ask
    book.insert_new_order(order);

    //cancel the bid
    assert_eq!(book.cancel_order(OrderId::new(1)), true);

    //confirm the ask is untouched
    assert_eq!(book.get_best_ask(), Some(Price::new(110)));

    //construct second bid
    let order = Order::new(
        OrderId::new(3),
        Price::new(130),
        Quantity::new(5),
        Side::Buy,
    );

    //insert second bid
    book.insert_new_order(order);

    //cancel second bid
    assert_eq!(book.cancel_order(OrderId::new(3)), true);

    //confirm the ask is untouched
    assert_eq!(book.get_best_ask(), Some(Price::new(110)));
}
