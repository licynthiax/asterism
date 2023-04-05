pub trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next<'a>(&mut self) -> &mut Option<Self::Item<'a>>;
}
