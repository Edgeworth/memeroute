#![warn(rust_2018_idioms, clippy::all)]
#![feature(
    array_chunks,
    array_windows,
    bool_to_option,
    const_fn_trait_bound,
    destructuring_assignment,
    drain_filter,
    is_sorted,
    map_first_last,
    option_result_contains,
    stmt_expr_attributes,
    const_panic,
    trait_alias
)]

pub mod dsn;
pub mod model;
pub mod route;
