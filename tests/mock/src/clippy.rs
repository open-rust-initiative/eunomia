pub fn deny_lints() {
    // almost_swapped
    let mut _a = 1;
    let mut _b = 2;
    _a = _b;
    _b = _a;
    // out_of_bounds_indexing
    let x = [1, 2, 3, 4];
    // no-effect
    #[allow(unconditional_panic)]
    x[9];
    let _ = &x[2..9];
}

pub fn warning_lints() {
    // bool_comparison
    let flag: bool = true;
    if flag == true {}
    // comparison_to_empty
    let s = "";
    if s == "" {}
    // double_neg
    #[allow(unused_mut)]
    let mut x = 3;
    let _ = --x;
}

pub fn allow_lints() {
    // float_cmp (pedantic)
    let x = 1.23456;
    let y = 1.23457;
    if y != x { }
    // unwrap_used
    let o = Some(1);
    let _ = o.unwrap();
}
