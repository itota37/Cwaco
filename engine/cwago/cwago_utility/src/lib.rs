// -------------------------
//
// Cwago.
//
// cwago/cwago_utility/src/lib.rs
// (C) 2022 CwagoCommunity.
//
//! cwago_utilityライブラリのメインファイルです。
// =========================

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

