
class Algorithm {
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