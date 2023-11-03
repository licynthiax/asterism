use std::iter::Iterator;

pub trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;

    /* fn filter<F>(self, function: F) -> Filter<Self>
    where
        F: Fn(&<Self as LendingIterator>::Item<'_>) -> bool + 'static,
        Self: std::marker::Sized,
    {
        Filter {
            iter: self,
            function: Box::new(function),
        }
    } */

    fn zip_lending<'item, I>(self, other_iter: I) -> ZipLending<Self, I>
    where
        I: LendingIterator,
        ZipLending<Self, I>: 'item,
        Self: std::marker::Sized,
    {
        ZipLending {
            iter_fst: self,
            iter_snd: other_iter,
        }
    }

    fn zip<'item, I>(self, other_iter: I) -> Zip<Self, I>
    where
        I: Iterator,
        Zip<Self, I>: 'item,
        Self: std::marker::Sized,
    {
        Zip {
            iter_fst: self,
            iter_snd: other_iter,
        }
    }

    fn enumerate<'item>(self) -> Enumerate<Self>
    where
        Enumerate<Self>: 'item,
        Self: std::marker::Sized,
    {
        Enumerate {
            iter: self,
            count: 0,
        }
    }
}

/* pub struct Filter<I: LendingIterator> {
    iter: I,
    function: Box<dyn Fn(&I::Item<'_>) -> bool>,
}

impl<I: LendingIterator> LendingIterator for Filter<I> {
    type Item<'a> = I::Item<'a> where I: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.iter.next().filter(self.function.as_mut())
    }
} */

pub struct ZipLending<I: LendingIterator, J: LendingIterator> {
    iter_fst: I,
    iter_snd: J,
}

impl<I: LendingIterator, J: LendingIterator> LendingIterator for ZipLending<I, J> {
    type Item<'a> = (I::Item<'a>, J::Item<'a>) where I: 'a, J: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let fst = self.iter_fst.next();
        let snd = self.iter_snd.next();

        if let Some(fst) = fst {
            snd.map(|snd| (fst, snd))
        } else {
            None
        }
    }
}

pub struct Zip<I: LendingIterator, J: Iterator> {
    iter_fst: I,
    iter_snd: J,
}

impl<I: LendingIterator, J: Iterator> LendingIterator for Zip<I, J> {
    type Item<'a> = (I::Item<'a>, J::Item) where I: 'a, J: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let fst = self.iter_fst.next();
        let snd = self.iter_snd.next();

        if let Some(fst) = fst {
            snd.map(|snd| (fst, snd))
        } else {
            None
        }
    }
}

pub struct Enumerate<I: LendingIterator> {
    iter: I,
    count: usize,
}

impl<I: LendingIterator> LendingIterator for Enumerate<I> {
    type Item<'a> = (usize, I::Item<'a>) where I: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let count = self.count;
        self.count += 1;
        self.iter.next().map(|item| (count, item))
    }
}
