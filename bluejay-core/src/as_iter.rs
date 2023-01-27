pub trait AsIter {
    type Item;
    type Iterator<'a>: Iterator<Item=&'a Self::Item> where Self: 'a;

    fn iter<'a>(&'a self) -> Self::Iterator<'a>;
}
