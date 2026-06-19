extern crate objc_test_utils;

use {Block, RcBlock};

pub fn get_int_block_with(i: i32) -> RcBlock<(), i32> {
    unsafe {
        let ptr = objc_test_utils::get_int_block_with(i);
        // SAFETY: objc_test_utils guarantees this returns a valid block pointer
        // with +1 ownership suitable for RcBlock::new in test environments.
        assert!(!ptr.is_null(), "objc_test_utils::get_int_block_with returned null");
        RcBlock::new(ptr as *mut _)
    }
}

pub fn get_add_block_with(i: i32) -> RcBlock<(i32,), i32> {
    unsafe {
        let ptr = objc_test_utils::get_add_block_with(i);
        // SAFETY: objc_test_utils guarantees this returns a valid block pointer
        // with +1 ownership suitable for RcBlock::new in test environments.
        assert!(!ptr.is_null(), "objc_test_utils::get_add_block_with returned null");
        RcBlock::new(ptr as *mut _)
    }
}

pub fn invoke_int_block(block: &Block<(), i32>) -> i32 {
    let ptr = block as *const _;
    unsafe {
        // SAFETY: Test utility API expects a mutable raw pointer type, but this
        // helper assumes the callee does not mutate through it.
        objc_test_utils::invoke_int_block(ptr as *mut _)
    }
}

pub fn invoke_add_block(block: &Block<(i32,), i32>, a: i32) -> i32 {
    let ptr = block as *const _;
    unsafe {
        // SAFETY: Test utility API expects a mutable raw pointer type, but this
        // helper assumes the callee does not mutate through it.
        objc_test_utils::invoke_add_block(ptr as *mut _, a)
    }
}
