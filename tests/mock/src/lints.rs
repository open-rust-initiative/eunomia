pub fn deny_lints() {
    // clippy::almost_swapped
    let mut _a = 1;
    let mut _b = 2;
    _a = _b;
    _b = _a;
    // clippy::out_of_bounds_indexing
    let x = [1, 2, 3, 4];
    let _ = &x[2..9];
}

pub fn warning_lints() {
    // clippy::bool_comparison
    let flag: bool = true;
    if flag == true {}
    // clippy::comparison_to_empty
    let s = "";
    if s == "" {}
    // clippy::double_neg
    #[allow(unused_mut)]
    let mut x = 3;
    let _ = --x;
}

pub fn allow_lints() {
    // clippy::float_cmp (pedantic)
    let x = 1.23456;
    let y = 1.23457;
    if y != x { }
    // clippy::unwrap_used
    let o = Some(1);
    let _ = o.unwrap();
}

// dead_code
fn dead_fn() { }

pub struct S;
pub fn rustc_warning_lints() {
    // unused-variables
    let x = 5;
    // while-true
    while true { }
    // private-in-public
    struct Priv;
    impl S {
        pub fn f(_: Priv) { }
    }
}

pub fn rustc_allow_lints() {
    // non-ascii-idents
    let _变量 = 1;
    // unsafe-code;
    let _ = unsafe {
        let p = std::ptr::null::<i32>();
        *p
    };
}

// unused_lifetimes
pub fn rustc_allow_lint_unused_lifetimes<'a>() { }
