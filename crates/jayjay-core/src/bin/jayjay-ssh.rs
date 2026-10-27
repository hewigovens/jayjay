fn main() {
    if let Err(error) = jayjay_core::ssh::serve() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
