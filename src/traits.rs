pub(crate) trait Randomizable<T> {
    fn get_random(&self) -> Option<T>;
}

impl<T> Randomizable<T> for Vec<T>
where
    T: Clone,
{
    #[inline]
    fn get_random(&self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let idx = rand::random::<usize>() % self.len();
            Some(self[idx].clone())
        }
    }
}
