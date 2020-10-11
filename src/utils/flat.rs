use std::iter::IntoIterator;

pub fn flat<T>(it: impl IntoIterator<Item = impl IntoIterator<Item = T>>) -> Vec<T> {
    let it_a = it;
    let mut r = vec![];
    for it_b in it_a {
        for elem in it_b {
            r.push(elem);
        }
    }

    r
}
