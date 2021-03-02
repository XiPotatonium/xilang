/* UTF-8 is supported
 * Author: Xi
 */
// Compound Test
mod demo;

class Program {
    fn main() {
        let d = crate::demo::Demo::new(1, 24);
        let a = d.foo(6);               // 32
        let d: i32 = gcd(a, d.value);   // gcd(32, 24)
        d;
    }

    fn gcd(a: i32, b: i32) -> i32 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }
}
