fn main() {
    print!(
        "{}",
        include_str!(concat!(env!("OUT_DIR"), "/Localized.swift"))
    );
}
