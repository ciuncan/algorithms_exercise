pub(crate) trait Randomizable<T> {
    fn get_random(&self) -> Option<T>;
}

impl<T> Randomizable<T> for Vec<T>
where
    T: Clone,
{
    #[inline]
    fn get_random(&self) -> Option<T> {
        if self.len() > 0 {
            let idx = rand::random::<usize>() % self.len();
            Some(self[idx].clone())
        } else {
            None
        }
    }
}
