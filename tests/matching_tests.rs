use orderbook::orderbook::OrderBook;
use orderbook::types::{Order, OrderId, Price, Quantity, Side, Trade};

#[test]
fn test_full_fill_both_sides() {
    //create fresh orderbook
    let mut orderbook = OrderBook::new();

    //resting sell order
    let resting = Order::new(
        OrderId::new(1),
        Price::new(100),
        Quantity::new(50),
        Side::Sell,
    );

    orderbook.insert_new_order(resting);

    //incoming buy order that exactly matches resting order
    let incoming = Order::new(
        OrderId::new(2),
        Price::new(100),
        Quantity::new(50),
        Side::Buy,
    );

    let (trades, leftover) = orderbook.process_order(incoming);

    //expect a single trade for the full quantity at the resting order price
    let expected_trade = Trade::new(
        Price::new(100),
        Quantity::new(50),
        OrderId::new(2),
        OrderId::new(1),
    );

    assert_eq!(trades, vec![expected_trade]);

    //incoming order fully filled, nothing leftover
    assert_eq!(leftover, Quantity::new(0));

    //price level should be gone entirely on both sides
    assert_eq!(orderbook.get_best_ask(), None);
    assert_eq!(orderbook.get_best_bid(), None);

    //resting order should no longer be cancellabe as it was consumed
    assert_eq!(orderbook.cancel_order(OrderId::new(1)), false);
}

#[test]
fn test_partial_fill_resting_side() {
    //create fresh orderbook
    let mut orderbook = OrderBook::new();

    //resting sell order
    let resting = Order::new(
        OrderId::new(1),
        Price::new(100),
        Quantity::new(100),
        Side::Sell,
    );

    orderbook.insert_new_order(resting);

    //incoming buy order smaller than resting
    let incoming = Order::new(
        OrderId::new(2),
        Price::new(100),
        Quantity::new(40),
        Side::Buy,
    );

    let (trades, leftover) = orderbook.process_order(incoming);

    let expected_trade = Trade::new(
        Price::new(100),
        Quantity::new(40),
        OrderId::new(2),
        OrderId::new(1),
    );

    assert_eq!(trades, vec![expected_trade]);

    //incoming order fully filled
    assert_eq!(leftover, Quantity::new(0));

    //resting order should still be sitting at the price level with 60 left
    assert_eq!(orderbook.get_best_ask(), Some(Price::new(100)));

    //prove the remaining quantity is exactly 60
    let followup = Order::new(
        OrderId::new(3),
        Price::new(100),
        Quantity::new(60),
        Side::Buy,
    );

    let (followup_trades, followup_leftover) = orderbook.process_order(followup);

    let expected_followup_trade = Trade::new(
        Price::new(100),
        Quantity::new(60),
        OrderId::new(3),
        OrderId::new(1),
    );

    assert_eq!(followup_trades, vec![expected_followup_trade]);
    assert_eq!(followup_leftover, Quantity::new(0));

    //the level should now be completely gone
    assert_eq!(orderbook.get_best_ask(), None);
}

#[test]
fn test_partial_fill_incoming_rests() {
    //create fresh orderbook
    let mut orderbook = OrderBook::new();

    //resting sell order
    let resting = Order::new(
        OrderId::new(1),
        Price::new(100),
        Quantity::new(40),
        Side::Sell,
    );

    orderbook.insert_new_order(resting);

    //incoming buy bigger than resting order
    let incoming = Order::new(
        OrderId::new(2),
        Price::new(100),
        Quantity::new(100),
        Side::Buy,
    );

    let (trades, leftover) = orderbook.process_order(incoming);

    let expected_trade = Trade::new(
        Price::new(100),
        Quantity::new(40),
        OrderId::new(2),
        OrderId::new(1),
    );

    assert_eq!(trades, vec![expected_trade]);

    //60 units of the incoming order remained
    assert_eq!(leftover, Quantity::new(60));

    //resting sell level is fully consumed and gone
    assert_eq!(orderbook.get_best_ask(), None);

    //leftover is not auto inserted so best bid should be empty
    assert_eq!(orderbook.get_best_bid(), None);

    //build new order from leftover quantity and rest it
    let resting_leftover = Order::new(OrderId::new(2), Price::new(100), leftover, Side::Buy);

    orderbook.insert_new_order(resting_leftover);

    //it should show up as the best bid
    assert_eq!(orderbook.get_best_bid(), Some(Price::new(100)));

    //it should be cancellable as it is truly resting
    assert_eq!(orderbook.cancel_order(OrderId::new(2)), true);
    assert_eq!(orderbook.get_best_bid(), None);
}

#[test]
fn test_multi_level_sweep() {
    //create fresh orderbook
    let mut orderbook = OrderBook::new();

    //two resting sell levels
    let level_one = Order::new(
        OrderId::new(1),
        Price::new(100),
        Quantity::new(30),
        Side::Sell,
    );

    orderbook.insert_new_order(level_one);

    let level_two = Order::new(
        OrderId::new(2),
        Price::new(101),
        Quantity::new(30),
        Side::Sell,
    );

    orderbook.insert_new_order(level_two);

    //incoming buy order priced high enough to cross both levels
    let incoming = Order::new(
        OrderId::new(3),
        Price::new(101),
        Quantity::new(50),
        Side::Buy,
    );

    let (trades, leftover) = orderbook.process_order(incoming);

    let expected_first_trade = Trade::new(
        Price::new(100),
        Quantity::new(30),
        OrderId::new(3),
        OrderId::new(1),
    );

    let expected_second_trade = Trade::new(
        Price::new(101),
        Quantity::new(20),
        OrderId::new(3),
        OrderId::new(2),
    );

    assert_eq!(trades, vec![expected_first_trade, expected_second_trade]);

    //incoming order fully filled
    assert_eq!(leftover, Quantity::new(0));

    //first level entirely gone
    assert_eq!(orderbook.get_best_ask(), Some(Price::new(101)));

    //prove second level has 10 units remaining
    let followup = Order::new(
        OrderId::new(4),
        Price::new(101),
        Quantity::new(10),
        Side::Buy,
    );

    let (followup_trades, followup_leftover) = orderbook.process_order(followup);

    let expected_followup_trade = Trade::new(
        Price::new(101),
        Quantity::new(10),
        OrderId::new(4),
        OrderId::new(2),
    );

    assert_eq!(followup_trades, vec![expected_followup_trade]);
    assert_eq!(followup_leftover, Quantity::new(0));

    //book should be completely empty on the ask side
    assert_eq!(orderbook.get_best_ask(), None);
}

#[test]
fn test_price_time_priority_within_level() {
    //create fresh orderbook
    let mut orderbook = OrderBook::new();

    //two resting sell orders at same price
    let first_in = Order::new(
        OrderId::new(1),
        Price::new(100),
        Quantity::new(20),
        Side::Sell,
    );

    orderbook.insert_new_order(first_in);

    let second_in = Order::new(
        OrderId::new(2),
        Price::new(100),
        Quantity::new(20),
        Side::Sell,
    );

    orderbook.insert_new_order(second_in);

    //incoming buy order that partially drains the level
    let incoming = Order::new(
        OrderId::new(3),
        Price::new(100),
        Quantity::new(25),
        Side::Buy,
    );

    let (trades, leftover) = orderbook.process_order(incoming);

    //order placed first should get filled first and completely
    let expected_first_trade = Trade::new(
        Price::new(100),
        Quantity::new(20),
        OrderId::new(3),
        OrderId::new(1),
    );

    let expected_second_trade = Trade::new(
        Price::new(100),
        Quantity::new(5),
        OrderId::new(3),
        OrderId::new(2),
    );

    assert_eq!(trades, vec![expected_first_trade, expected_second_trade]);
    assert_eq!(leftover, Quantity::new(0));

    //level should still exist since 2nd order has 15 units left
    assert_eq!(orderbook.get_best_ask(), Some(Price::new(100)));

    //order 1 was fully consumed so cancelling it should fail
    assert_eq!(orderbook.cancel_order(OrderId::new(1)), false);

    //order 2 should be cancellable as it's still resting
    assert_eq!(orderbook.cancel_order(OrderId::new(2)), true);

    //the level should be gone
    assert_eq!(orderbook.get_best_ask(), None);
}

#[test]
fn test_no_cross() {
    //create fresh orderbook
    let mut orderbook = OrderBook::new();

    //resting sell order
    let resting = Order::new(
        OrderId::new(1),
        Price::new(100),
        Quantity::new(20),
        Side::Sell,
    );

    orderbook.insert_new_order(resting);

    //incoming buy placed below best ask so it shouldn't cross
    let incoming = Order::new(
        OrderId::new(2),
        Price::new(99),
        Quantity::new(10),
        Side::Buy,
    );

    //check the crossing logic directly
    assert_eq!(
        orderbook.check_if_order_crosses(Price::new(99), Side::Buy),
        false
    );

    let (trades, leftover) = orderbook.process_order(incoming);

    //no trade should have happened at all
    assert_eq!(trades, Vec::new());

    //full incoming quantity should be untouched
    assert_eq!(leftover, Quantity::new(10));

    //resting ask is completely unaffected
    assert_eq!(orderbook.get_best_ask(), Some(Price::new(100)));

    //bid side should be empty
    assert_eq!(orderbook.get_best_bid(), None);
}
