# 作业2的问题

> 老师，您好，第二课的作业我有一个地方始终没有找到原因，所以想让您帮忙解答一下。
> 首先，我的代码地址是：
> https://github.com/luckychacha/substrate-node-template/tree/v3.0.0-Learning
> 分支：
> v3.0.0-Learning
> 我现在疑惑的点在于这个测试用例：

``` rust

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

```

> 我的程序中，每只猫创建时，用户需要质押 100 ，转移时，原持有者返还质押 100，新持有者质押 100。
> 我这个测试用例的前提是：

``` rust

pallet_balances::GenesisConfig::<Test> {
    balances: vec![(1, 200), (2, 100), (3, 66), (4, 99), (5, 66), (6, 1000), (7, 266)],
}

```

> 用户 1 的余额是 200，足够创建一只猫，用户 3 余额 66 不足 100，所以猫从用户 1 转移给用户 3 的时候，我断言用户 3 质押时会抛出异常：Error::<T>::ReserveFailed。我 github 上的代码执行测试用例是可以通过的，第 8 - 11 行直接先判断 新用户 是否足够质押，第 16 行才是再返还 原用户 的质押。代码截图如下：

```rust
fn transfer_kitty(
    owner_id: &T::AccountId,
    new_owner_id: &T::AccountId,
    kitty_id: T::KittyIndex,
) -> DispatchResult {
    ensure!(Some(owner_id.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotKittyOwner);

    ensure!(
        T::Currency::can_reserve(&new_owner_id, T::CreateKittyReserve::get()),
        Error::<T>::ReserveFailed
    );
    T::Currency::reserve(&new_owner_id, T::CreateKittyReserve::get())
        .map_err(|_| {
            Error::<T>::ReserveFailed
        })?;
    T::Currency::unreserve(&owner_id, T::CreateKittyReserve::get());


    Owner::<T>::insert(kitty_id, Some(new_owner_id.clone()));

    // if transfer action from sell or has set price before
    // need to remove price of kitty price.
    match PriceOf::<T>::get(kitty_id.clone()) {
        Some(_) => PriceOf::<T>::remove(kitty_id.clone()),
        None => ()
    }

    Ok(())
}

```

> 执行 cargo test transfer_kitty_failed_when_reserve_failed
>  测试通过，命令行输出：
> 
> running 1 test
>   test tests::transfer_kitty_failed_when_reserve_failed ... ok

> 这时候，我偶然发现，调整代码顺序，新代码截图如下：
> 第 9 行先对原持有者解除质押， 10 - 12 行对新用户的余额进行校验，判断是否足够质押，不够就抛出异常 Error::<T>::ReserveFailed，但是此时，测试用例就无法通过。

```rust

fn transfer_kitty(
    owner_id: &T::AccountId,
    new_owner_id: &T::AccountId,
    kitty_id: T::KittyIndex,
) -> DispatchResult {
    ensure!(Some(owner_id.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotKittyOwner);

    T::Currency::unreserve(&owner_id, T::CreateKittyReserve::get());
    ensure!(
        T::Currency::can_reserve(&new_owner_id, T::CreateKittyReserve::get()),
        Error::<T>::ReserveFailed
    );
    T::Currency::reserve(&new_owner_id, T::CreateKittyReserve::get())
        .map_err(|_| {
            Error::<T>::ReserveFailed
        })?;


    Owner::<T>::insert(kitty_id, Some(new_owner_id.clone()));

    // if transfer action from sell or has set price before
    // need to remove price of kitty price.
    match PriceOf::<T>::get(kitty_id.clone()) {
        Some(_) => PriceOf::<T>::remove(kitty_id.clone()),
        None => ()
    }

    Ok(())
}

```

> 测试用例的输出结果如下：
>   ---- tests::transfer_kitty_failed_when_reserve_failed stdout ----
> thread 'tests::transfer_kitty_failed_when_reserve_failed' panicked at 'assertion failed: `(left == right)`
>  left: `[116, 2, 105, 180, 117, 11, 108, 50, 86, 62, 138, 151, 235, 173, 107, 76, 78, 62, 171, 30, 228, 82, 67, 196, 222, 247, 28, 43, 166, 245, 206, 3]`,
> right: `[195, 30, 47, 19, 29, 206, 68, 149, 66, 72, 181, 183, 58, 253, 249, 213, 162, 189, 236, 2, 158, 5, 162, 22, 113, 47, 72, 109, 241, 142, 242, 36]`', pallets/kitties/src/tests.rs:65:9
> note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

> 测试的提示是左右两侧不相等，不过输出的内容不可读，我无法了解到底左右分别对应的内容是什么，我尝试着把输出作为一个 vec![u8] 来解析成字符去解读也是乱码的，不知道老师有没有更好的建议来解读这个奇怪的输出。
> 然后我做了这个尝试，就是修改了测试用例中右侧的错误，换成了一个其他的错误【InvalidKittyPrice】，代码截图如下：

```rust

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
            Error::<Test>::InvalidKittyPrice
        );
    })
}

```

> 测试用例运行输出结果如下：
> ---- tests::transfer_kitty_failed_when_reserve_failed stdout ----
> thread 'tests::transfer_kitty_failed_when_reserve_failed' panicked at 'assertion failed: `(left == right)`
>  left: `Err(Module { index: 1, error: 4, message: Some("ReserveFailed") })`,
> right: `Err(Module { index: 1, error: 5, message: Some("InvalidKittyPrice") })`', pallets/kitties/src/tests.rs:65:9
> note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

> 此时的报错是可读的，而且可以看出左侧代码执行的结果确实抛出了异常 Error::<T>::ReserveFailed，和右侧我随意改为别的报错不匹配，那么为什么改之前的测试用例没通过呢？

>后面我还进行了一种尝试，修改了测试用例的代码，截图如下：
> 我让变量 a = 转移函数返回的结果，断言 a 就是 Error::<T>::ReserveFailed， 此时测试用例也是可以通过的，代码截图如下：

```rust

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

```
> 测试用例运行结果如下：
> 
> running 1 test
> test tests::transfer_kitty_failed_when_reserve_failed ... ok

> 这个就让我怀疑是否是处理结果不当导致的呢？

> 所以目前遇到的 2 个问题如下：
> 1.把转移代码中的逻辑改为：先返还原持有者的质押，再对信持有者的余额进行校验，此时测试用例无法通过。
> 2.测试用例无法通过，但是左右两侧输出的内容不知道该用哪种方式翻译成可读的语言，而且把右侧断言的异常换为其他异常，左侧得到的异常其实与我改之前的右侧的异常是相同的，如果赋值给一个临时变量，临时变量其实与断言的错误是一致的。

