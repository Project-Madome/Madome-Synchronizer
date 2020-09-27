pub fn nozomi() -> String {
    String::from("https://ltn.hitomi.la/index-korean.nozomi")
}

pub fn galleries(id: i32) -> String {
    format!("https://hitomi.la/galleries/{}.html", id)
}
