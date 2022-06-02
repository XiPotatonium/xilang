interface IIterable<T> {
    fn len(self) -> usize;
    fn iter(self) -> IIterator<T>;

    fn foo<I>(self);
    fn bar<I>(self);
}

interface IIterator<T> {
    fn next(self) -> T;
    fn has_nest(self) -> bool;
}

struct List: IIterable<T> {
    data: T[],

    fn create(n: usize) -> List<T> {
        Self::foo<usize>();
        new List<T> {
            data: new T[n]
        }
    }

    // overload is allowed
    fn len(self) -> usize {
        self.data.len()
    }

    fn iter(self) -> ListIter<T> {
        new ListIter<T> {
            arr: self,
            idx: 0
        }
    }

    fn foo<M>() { }

    // bar() is not implemented. But that is OK.
    // Compilation should pass, but calling of .bar() will panic
}

struct ListIter<T>: IIterator<T> {
    arr: List<T>,
    idx: usize,

    fn next(self) -> T {
        let ret = self.arr.data[self.idx];
        self.idx = self.idx + 1;
        ret
    }

    fn has_nest(self) -> bool {
        self.idx < N
    }
}
