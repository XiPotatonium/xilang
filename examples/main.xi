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

        let res = is_prime(12);
        let res = is_prime(13);
    }

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
}
