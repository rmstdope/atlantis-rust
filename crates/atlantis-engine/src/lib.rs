//! Pure Rust core for the Atlantis game engine.

pub mod output;
pub mod version;
pub mod text;

#[cfg(test)]
mod version_tests {
    use crate::version::{
        AtlVersion, CURRENT_ATL_VER, NEW_ORIGINS_RULESET_VERSION, STANDARD_RULESET_VERSION,
    };

    #[test]
    fn version_components_round_trip_through_packed_representation() {
        assert_eq!(CURRENT_ATL_VER.packed(), 0x0005_0205);
        assert_eq!(CURRENT_ATL_VER.major(), 5);
        assert_eq!(CURRENT_ATL_VER.minor(), 2);
        assert_eq!(CURRENT_ATL_VER.patch(), 5);
        assert_eq!(STANDARD_RULESET_VERSION.packed(), 0x0002_0000);
        assert_eq!(NEW_ORIGINS_RULESET_VERSION.packed(), 0x0003_0000);
        assert_eq!(AtlVersion::new(5, 1, 0).packed(), 0x0005_0100);
    }

    #[test]
    fn version_string_marks_only_even_minor_versions_as_beta() {
        assert_eq!(CURRENT_ATL_VER.to_string(), "5.2.5 (beta)");
        assert_eq!(AtlVersion::new(5, 1, 0).to_string(), "5.1.0");
    }
}

#[cfg(test)]
mod output_tests {
    use crate::output::EngineOutput;
    use crate::version::{
        write_startup_banner, CURRENT_ATL_VER, NEW_ORIGINS_RULESET_NAME,
        NEW_ORIGINS_RULESET_VERSION, STANDARD_RULESET_NAME, STANDARD_RULESET_VERSION,
    };

    #[test]
    fn output_lines_and_dots_match_cpp_streaming_behavior() {
        let mut output = EngineOutput::new();
        output.write_line("Starting");
        output.write_dot();
        output.write_line(" done");

        assert_eq!(output.as_str(), "Starting\n. done\n");
        assert_eq!(output.into_string(), "Starting\n. done\n");
    }

    #[test]
    fn startup_banners_match_both_supported_rulesets() {
        let mut standard = EngineOutput::new();
        write_startup_banner(
            &mut standard,
            STANDARD_RULESET_NAME,
            STANDARD_RULESET_VERSION,
        );
        assert_eq!(
            standard.as_str(),
            "Atlantis Engine Version: 5.2.5 (beta)\n\
             Wyreth, Version: 2.0.0 (beta)\n\
             \n"
        );

        let mut new_origins = EngineOutput::new();
        write_startup_banner(
            &mut new_origins,
            NEW_ORIGINS_RULESET_NAME,
            NEW_ORIGINS_RULESET_VERSION,
        );
        assert_eq!(
            new_origins.as_str(),
            format!(
                "Atlantis Engine Version: {CURRENT_ATL_VER}\n\
                 NewOrigins, Version: 3.0.0 (beta)\n\
                 \n"
            )
        );
    }
}
