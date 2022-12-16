// -------------------------
//
// Cwago.
//
// cwago/cwago_utility/src/lib.rs
// (C) 2022 CwagoCommunity.
//
//! cwago_utilityライブラリのメインファイルです。
// =========================

pub mod mem;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mem() {
        println!("test");
        //assert_eq!(result, 4);
    }
}