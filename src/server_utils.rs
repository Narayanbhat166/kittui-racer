use rnglib;

pub fn generate_name() -> String {
    let random_name_generator = rnglib::RNG::from(&rnglib::Language::Fantasy);

    format!(
        "{} {}",
        random_name_generator.generate_name(),
        random_name_generator.generate_name()
    )
}
