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

const START: i32 = 24;

// entry of the program
fn main() {
    let i = START;
    while !is_prime(i) {
        sys::println(a);
        let d: i32 = gcd(a, d.value);
        sys::println(d);

        i = i + 1;
    }
}
