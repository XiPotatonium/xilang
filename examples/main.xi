/* 已经支持UTF-8
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

// entry of the program
fn main() {
}
