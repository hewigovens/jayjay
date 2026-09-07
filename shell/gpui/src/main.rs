fn main() {
    std::process::exit(jayjay_gpui::startup::run(
        &std::env::args().skip(1).collect::<Vec<_>>(),
    ));
}
