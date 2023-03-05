pub trait AsIter {
    type Item;
    type Iterator<'a>: Iterator<Item = &'a Self::Item>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iterator<'_>;

    fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }
}

impl<T> AsIter for Vec<T> {
    type Item = T;
    type Iterator<'a> = std::slice::Iter<'a, Self::Item> where T: 'a;

    fn iter(&self) -> Self::Iterator<'_> {
        self.as_slice().iter()
    }
}
