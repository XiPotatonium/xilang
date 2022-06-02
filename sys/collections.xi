

interface IIterable<T> {
    fn len(self) -> usize;
    fn iter(self) -> IIterator<T>;
}

interface IIterator<T> {
    fn next(self) -> opt<T>;
}

struct List: IIterable<T> {
    data: T[],

    fn ctor(n: usize) -> List<T> {
        Self::foo<usize>();
        new List<T> {
            data: new T[n]
        }
    }

    fn len(self) -> usize {
        self.data.len()
    }

    fn iter(self) -> ListIter<T> {
        new ListIter<T> {
            arr: self,
            idx: 0
        }
    }
}

struct ListIter<T>: IIterator<T> {
    arr: List<T>,
    idx: usize,

    fn next(self) -> opt<T> {
        if self.idx < self.arr.len() {
            self.idx = self.idx + 1;
            opt<T>::Some(self.arr.data[self.idx - 1])
        } else {
            opt<T>::None()
        }
    }

    fn has_nest(self) -> bool {
        self.idx < N
    }
}
