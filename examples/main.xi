/* UTF-8 is supported
 * Author: Xi
 */
// Compound Test
mod demo;

class Program {
    fn main() {
        let d: demo::Demo = crate::demo::Demo::create(1, 24);
        let a = d.foo(6);               // 32
        let d: i32 = demo::algo::Algorithm::gcd(a, d.value);    // gcd(32, 24)

        let res = demo::algo::Algorithm::is_prime(12);
        let res = demo::algo::Algorithm::is_prime(13);
    }
}
