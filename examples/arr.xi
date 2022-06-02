use sys;

struct MyPair {
    key: i32,
    val: str,

    fn create(key: i32, value: str) -> MyPair {
        new Self { key, val: value }
    }
}

fn main() {
    sys::println("Array Test:");

    let arr = new MyPair[3];
    let i = 0;
    while i < arr.len {
        arr[i] = MyPair::create(i, "MyPairs");
        arr[i].val = "MyPair";
        i = i + 1;
    }
    i = i - 1;
    while i >= 0 {
        arr[i].print();
        sys::print(' ');
        i = i - 1;
    }
    sys::print('\n');
}
