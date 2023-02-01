pub trait AsIter {
    type Item;
    type Iterator<'a>: Iterator<Item = &'a Self::Item>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iterator<'_>;
}
