use super::super::{DEFAULT_WRAP_COLS, MIN_WRAP_COLS, wrap_cols_for_width};

#[test]
fn wrap_cols_for_width_clamps_and_defaults() {
    assert_eq!(wrap_cols_for_width(0., 8.), DEFAULT_WRAP_COLS);
    assert_eq!(wrap_cols_for_width(800., 0.), DEFAULT_WRAP_COLS);
    assert_eq!(wrap_cols_for_width(40., 8.), MIN_WRAP_COLS);
    // 800 / 8 = 100 cells, minus 1 for trailing gutter padding.
    assert_eq!(wrap_cols_for_width(800., 8.), 99);
}
