pub fn get_ext(x: &str) -> Option<&str> {
    x.split('.').last()
}
