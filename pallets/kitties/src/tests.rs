use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_kitties_works() {
    new_test_ext().execute_with(|| {
        let account_id_1: u64 = 1;
        assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));
    })
}

#[test]
fn create_kitty_failed_when_balance_less_than_reserve() {
    new_test_ext().execute_with(|| {
        let account_id_5: u64 = 5;
        assert_noop!(
            KittiesModule::create(Origin::signed(account_id_5)),
            Error::<Test>::ReserveFailed
        );
    })
}

#[test]
fn create_kitty_failed_when_kitty_overflow() {
    new_test_ext().execute_with(|| {
        KittiesCount::<Test>::put(u32::max_value());

        let account_id_1: u64 = 1;
        assert_noop!(
            KittiesModule::create(Origin::signed(account_id_1)),
            Error::<Test>::KittiesCountOverflow
        );
    })
}

#[test]
fn transfer_kitty_works() {
    new_test_ext().execute_with(|| {
        let account_id_1: u64 = 1;
        let account_id_2: u64 = 2;
        assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));
        assert_ok!(KittiesModule::transfer(Origin::signed(account_id_1), account_id_2, 0));
    })
}

#[test]
fn transfer_kitty_failed_when_not_owner() {
    new_test_ext().execute_with(|| {
        let account_id_1: u64 = 1;
        let account_id_2: u64 = 2;
        let account_id_3: u64 = 3;
        assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));
        assert_noop!(
            KittiesModule::transfer(Origin::signed(account_id_3), account_id_2, 0),
            Error::<Test>::NotKittyOwner
        );
    })
}

#[test]
fn transfer_kitty_failed_when_reserve_failed() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(1)));
        assert_noop!(
            KittiesModule::transfer(
                Origin::signed(1),
                3,
                0,
            ),
            Error::<Test>::ReserveFailed
        );
    })
}

#[test]
fn breed_kitty_works() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesModule::create(Origin::signed(6)));
        assert_ok!(KittiesModule::create(Origin::signed(6)));
        assert_ok!(KittiesModule::breed(Origin::signed(6), 0, 1));
    })
}
// InvalidKittyIndex
// NotKittyOwner
#[test]
fn breed_kitty_failed_when_invalid_kitty_index() {
    new_test_ext().execute_with( || {
        assert_noop!(
            KittiesModule::breed(Origin::signed(6), 0, 1),
            Error::<Test>::InvalidKittyIndex
        );
    })
}

#[test]
fn breed_kitty_failed_when_parent_index_duplicated() {
    new_test_ext().execute_with( || {
        assert_noop!(
            KittiesModule::breed(Origin::signed(6), 0, 0),
            Error::<Test>::SameParentIndex
        );
    })
}

#[test]
fn breed_kitty_failed_when_not_kitty_owner() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesModule::create(Origin::signed(1)));
        assert_ok!(KittiesModule::create(Origin::signed(6)));
        assert_noop!(
            KittiesModule::breed(Origin::signed(6), 0, 1),
            Error::<Test>::NotKittyOwner
        );
    })
}

#[test]
fn breed_kitty_failed_when_not_balance_not_enough() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesModule::create(Origin::signed(7)));
        assert_ok!(KittiesModule::create(Origin::signed(7)));
        assert_noop!(
            KittiesModule::breed(Origin::signed(7), 0, 1),
            Error::<Test>::ReserveFailed
        );
    })
}

#[test]
fn breed_kitty_failed_when_overflow() {
    new_test_ext().execute_with( || {

        assert_ok!(KittiesModule::create(Origin::signed(6)));
        assert_ok!(KittiesModule::create(Origin::signed(6)));
        KittiesCount::<Test>::put(u32::max_value());
        assert_noop!(
            KittiesModule::breed(Origin::signed(6), 0, 1),
            Error::<Test>::KittiesCountOverflow
        );
    })
}


#[test]
fn sell_kitty_works() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesModule::create(Origin::signed(6)));
        assert_ok!(KittiesModule::sell(Origin::signed(6), 0, 1));
    })
}

#[test]
fn sell_kitty_failed_when_price_invalid() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesModule::create(Origin::signed(6)));
        assert_noop!(
            KittiesModule::sell(Origin::signed(6), 0, 0),
            Error::<Test>::InvalidKittyPrice
        );
    })
}

#[test]
fn sell_kitty_failed_when_not_owner() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesModule::create(Origin::signed(6)));
        assert_noop!(
            KittiesModule::sell(Origin::signed(7), 0, 0),
            Error::<Test>::NotKittyOwner
        );
    })
}

#[test]
fn buy_kitty_works() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesModule::create(Origin::signed(6)));
        assert_ok!(KittiesModule::sell(Origin::signed(6), 0, 1));
        assert_ok!(KittiesModule::buy(Origin::signed(7), 6, 0));
    })
}

#[test]
fn buy_kitty_failed_when_kitty_not_for_sale() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesModule::create(Origin::signed(6)));
        assert_noop!(
            KittiesModule::buy(Origin::signed(7), 6, 0),
            Error::<Test>::KittyNotForSale
        );
    })
}

#[test]
fn buy_kitty_failed_when_not_kitty_owner() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesModule::create(Origin::signed(6)));
        assert_ok!(KittiesModule::sell(Origin::signed(6), 0, 1));
        assert_noop!(
            KittiesModule::buy(Origin::signed(7), 1, 0),
            Error::<Test>::NotKittyOwner
        );
    })
}

#[test]
fn buy_kitty_failed_when_balance_not_enough() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesModule::create(Origin::signed(6)));
        assert_ok!(KittiesModule::sell(Origin::signed(6), 0, 600));
        assert_noop!(
            KittiesModule::buy(Origin::signed(1), 6, 0),
            Error::<Test>::BalanceNotEnough
        );
    })
}

#[test]
fn buy_kitty_failed_when_already_have_the_kitty() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesModule::create(Origin::signed(6)));
        assert_ok!(KittiesModule::sell(Origin::signed(6), 0, 1));
        assert_noop!(
            KittiesModule::buy(Origin::signed(6), 6, 0),
            Error::<Test>::KittyAlreadyHave
        );
    })
}