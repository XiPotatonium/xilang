/* 注释已经支持UTF-8
 * Author: shwu
 */

use sys;


fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn is_prime(n: i32) -> bool {
    if n == 1 {
        return false;
    }

    let i = 2;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }

        i = i + 1;
    }

    true
}

struct MyPair {
    let key: i32;
    let val: string;

    fn create(key: i32, value: string) -> MyPair {
        new Self { key, val: value }
    }
}

fn str_test() {
    sys::println("String Test:");
    let s: string = "Hello world!";
    sys::print(s.len());
    sys::println(s);
}

fn arr_test() {
    sys::println("Array Test:");

    let struct_arr = new MyPair[3];
    let i = 0;
    while i < struct_arr.len {
        struct_arr[i] = MyPair::create(i, "MyPairs");
        struct_arr[i].val = "MyPair";
        i = i + 1;
    }
    i = i - 1;
    while i >= 0 {
        struct_arr[i].print();
        sys::print(' ');
        i = i - 1;
    }
    sys::print('\n');
}

const START: i32 = 24;

fn logic_test() {
    let i = START;
    while !is_prime(i) {
        sys::println(a);
        let d: i32 = Algorithm::gcd(a, d.value);
        sys::println(d);

        i = i + 1;
    }
}

// entry of the program
fn main() {
    logic_test();
    str_test();
    arr_test();
}
