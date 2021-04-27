/* UTF-8 is supported
 * Author: Xi
 */
// Compound Test
mod demo;
mod IOHelper;

class Program: IOHelper::IOBase {
    fn main() {
        let prog = new Self();
        IOHelper::Helper::write_i32_ln(prog.hi_count);      // 0
        prog.hi();                                          // HI
        IOHelper::Helper::write_i32_ln(prog.hi_count);      // 1
        prog.hi();                                          // HI
        IOHelper::Helper::write_i32_ln(prog.hi_count);      // 2

        let d: demo::Demo = crate::demo::Demo::create(1, 24);
        let a = d.foo(6);               // 32
        IOHelper::Helper::write_i32_ln(a);
        let d: i32 = demo::algo::Algorithm::gcd(a, d.value);    // gcd(32, 24)
        IOHelper::Helper::write_i32_ln(d);

        let res = demo::algo::Algorithm::is_prime(12);
        let res = demo::algo::Algorithm::is_prime(13);
    }
}
