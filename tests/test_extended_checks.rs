//! EXT-01..EXT-50 extended verification matrix (shbt-power lineage).

use exotic_extended_audit::*;

macro_rules! ext_tests {
    ($($t:ident => $f:ident),* $(,)?) => {
        $(#[test]
        fn $t() {
            assert!($f(), "{} failed", stringify!($t));
        })*
    };
}

ext_tests! {
    ext_check_01 => ext_01, ext_check_02 => ext_02, ext_check_03 => ext_03,
    ext_check_04 => ext_04, ext_check_05 => ext_05, ext_check_06 => ext_06,
    ext_check_07 => ext_07, ext_check_08 => ext_08, ext_check_09 => ext_09,
    ext_check_10 => ext_10, ext_check_11 => ext_11, ext_check_12 => ext_12,
    ext_check_13 => ext_13, ext_check_14 => ext_14, ext_check_15 => ext_15,
    ext_check_16 => ext_16, ext_check_17 => ext_17, ext_check_18 => ext_18,
    ext_check_19 => ext_19, ext_check_20 => ext_20, ext_check_21 => ext_21,
    ext_check_22 => ext_22, ext_check_23 => ext_23, ext_check_24 => ext_24,
    ext_check_25 => ext_25, ext_check_26 => ext_26, ext_check_27 => ext_27,
    ext_check_28 => ext_28, ext_check_29 => ext_29, ext_check_30 => ext_30,
    ext_check_31 => ext_31, ext_check_32 => ext_32, ext_check_33 => ext_33,
    ext_check_34 => ext_34, ext_check_35 => ext_35, ext_check_36 => ext_36,
    ext_check_37 => ext_37, ext_check_38 => ext_38, ext_check_39 => ext_39,
    ext_check_40 => ext_40, ext_check_41 => ext_41, ext_check_42 => ext_42,
    ext_check_43 => ext_43, ext_check_44 => ext_44, ext_check_45 => ext_45,
    ext_check_46 => ext_46, ext_check_47 => ext_47, ext_check_48 => ext_48,
    ext_check_49 => ext_49, ext_check_50 => ext_50,
}
