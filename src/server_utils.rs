use rnglib;
pub mod fast_storage;
pub mod message_handlers;

pub fn generate_name() -> String {
    let random_name_generator = rnglib::RNG::from(&rnglib::Language::Fantasy);

    format!(
        "{} {}",
        random_name_generator.generate_name(),
        random_name_generator.generate_name()
    )
}

/// execute the function `func` after `time` seconds
pub async fn set_timeout<Fut>(time: u64, func: impl FnOnce() -> Fut)
where
    Fut: futures_util::Future<Output = ()> + Send,
{
    tokio::time::sleep(std::time::Duration::from_secs(time)).await;
    func().await;
}
