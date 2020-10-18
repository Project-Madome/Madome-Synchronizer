mod flat;
mod seperate;
mod text_store;

pub use flat::flat;
pub use seperate::seperate;
pub use text_store::TextStore;

pub trait VecUtil {
    type Item;

    fn seperate(self, by: usize) -> Vec<Vec<Self::Item>>;
}

pub trait Flat {
    type Item;

    fn flat(self) -> Vec<Self::Item>;
}

impl<T> VecUtil for Vec<T> {
    type Item = T;

    fn seperate(self, by: usize) -> Vec<Vec<Self::Item>> {
        seperate(self, by)
    }
}

impl<T> Flat for Vec<Vec<T>> {
    type Item = T;

    fn flat(self) -> Vec<Self::Item> {
        flat(self)
    }
}

pub trait IntoResultVec<T, E> {
    fn into_result_vec(self) -> Result<Vec<T>, E>;
}

impl<T, E> IntoResultVec<T, E> for Vec<Result<T, E>> {
    fn into_result_vec(self) -> Result<Vec<T>, E> {
        let mut r: Vec<T> = vec![];

        for elem in self {
            match elem {
                Ok(elem) => r.push(elem),
                Err(err) => return Err(err),
            }
        }

        Ok(r)
    }
}
