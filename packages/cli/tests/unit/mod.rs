// Unit tests module
// Links all unit test modules together

pub mod validators {
    pub mod llms_txt_comprehensive_test;
    pub mod llms_txt_edge_cases_test;
    pub mod consistency_test;
    pub mod consistency_comprehensive_test;
}

pub mod generators {
    pub mod llms_txt_generator_test;
    pub mod toon_test;
}

pub mod utils {
    pub mod config_test;
    pub mod mod_test;
}

// Command tests
mod commands {
    pub mod generate_additional_test;
    pub mod validate_additional_test;
    pub mod robots_test;
}
