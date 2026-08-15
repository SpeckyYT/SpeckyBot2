pub fn log_event(event: &str, details: impl std::fmt::Display) {
    println!("[EVENT] {event}: {details}");
}
