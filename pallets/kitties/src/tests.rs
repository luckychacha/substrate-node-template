use crate::{mock::*, Error};
use frame_support::{assert_err, assert_noop, assert_ok};

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
        let a = KittiesModule::transfer(
            Origin::signed(1),
            3,
            0,
        );
        assert_noop!(
            a,
            Error::<Test>::ReserveFailed
        );
    })
}
