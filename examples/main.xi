/* UTF-8 is supported
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
    loop {
        if i * i > n {
            break;
        }

        if n % i == 0 {
            return false;
        }

        i = i + 1;
    }

    true
}

class MyPair {
    static {
        Self::COUNT = 0;
    }

    static COUNT: i32;
    let key: i32;
    let val: string;

    fn create(value: string) -> MyPair {
        Self::create_with_key(Self::COUNT, value)
    }

    fn create_with_key(key: i32, value: string) -> MyPair {
        Self::COUNT = Self::COUNT + 1;
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
    loop {
        if i >= struct_arr.len {
            break;
        }
        struct_arr[i] = MyPair::create(i, "MyPairs");
        struct_arr[i].val = "MyPair";
        i = i + 1;
    }
    loop {
        i = i - 1;
        if i < 0 {
            break;
        }
        struct_arr[i].print();
        sys::print(' ');
    }
    sys::print('\n');

    let matrix: i32[][] = new i32[5][];
    i = 0;
    loop {
        if i >= matirx.len {
            break;
        }
        let row = new i32[3];
        let j = 0;
        loop {
            if j >= row.len {
                break;
            }
            row[j] = i * row.len + j;
            j = j + 1;
        }
        matirx[i] = row;
        i = i + 1;
    }
    i = 0;
    loop {
        if i >= matirx.len {
            break;
        }
        let j = 0;
        loop {
            if j >= matirx[i].len {
                break;
            }
            sys::print(matrix[i][j]);
            sys::print(' ');
            j = j + 1;
        }
        sys::print('\n');
        i = i + 1;
    }
}

fn logic_test() {
    sys::println("Basic Logic Test:");
    let d: Demo = new Demo { key: 1, value: 24 };
    let a = d.foo(6);               // 32
    sys::println(a);
    let d: i32 = Algorithm::gcd(a, d.value);    // gcd(32, 24)
    sys::println(d);
}

fn main() {
    logic_test();
    str_test();
    arr_test();
}
