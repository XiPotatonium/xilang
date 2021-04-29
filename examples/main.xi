/* UTF-8 is supported
 * Author: Xi
 */
// Compound Test
mod demo;
mod IOHelper;

class Base {
    virtual fn say(self) {
        std::IO::writeln(1000);
    }
}

class Derived: Base {
    override fn say(self) {
        std::IO::writeln(1001);
    }
}

class Program: IOHelper::IOBase {
    fn main() {
        let prog = new Self();
        std::IO::writeln(prog.hi_count);        // 0
        prog.hi();                              // HI
        std::IO::writeln(prog.hi_count);        // 1
        prog.hi();                              // HI
        std::IO::writeln(prog.hi_count);        // 2

        let d: demo::Demo = new crate::demo::Demo(1, 24);
        let a = d.foo(6);               // 32
        std::IO::writeln(a);
        let d: i32 = demo::algo::Algorithm::gcd(a, d.value);    // gcd(32, 24)
        std::IO::writeln(d);
        let d = new demo::Demo(30);
        std::IO::writeln(d.foo(4));     // 0 + 2 + 4 + 30

        let res = demo::algo::Algorithm::is_prime(12);
        let res = demo::algo::Algorithm::is_prime(13);
    }
}
