#[allow(clippy::eq_op)]
mod assert_eq {
    #[test]
    fn passes() {
        let a = "some value";
        ::pretty_assertions_sorted::assert_eq_sorted!(a, a);
    }
}
