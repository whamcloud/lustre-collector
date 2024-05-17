#[cfg(test)]
mod lustre_integration_tests {
    use std::path::PathBuf;

    use insta::assert_debug_snapshot;
    use lustre_collector::parse;
    use lustre_collector::utils::CommandMode;

    macro_rules! generate_test {
        ($name:ident, $version:expr, $mode:expr) => {
            #[test]
            fn $name() {
                let cargo_dir: &str = env!("CARGO_MANIFEST_DIR");
                let version: &str = $version;
                let path = PathBuf::from(format!("{cargo_dir}/cassettes/{version}/"));
                let mode = $mode;
                assert_debug_snapshot!(parse(&mode, &path));
            }
        };
    }

    generate_test!(test_lustre_ddn145, "2.14.0_ddn145", CommandMode::Play);
}
