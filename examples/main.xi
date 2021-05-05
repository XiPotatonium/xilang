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
    fn logic_test() {
        let d: demo::Demo = new crate::demo::Demo(1, 24);
        let a = d.foo(6);               // 32
        std::IO::writeln(a);
        let d: i32 = demo::algo::Algorithm::gcd(a, d.value);    // gcd(32, 24)
        std::IO::writeln(d);
        let d = new demo::Demo(30);
        std::IO::writeln(d.foo(4));     // 0 + 2 + 4 + 30
    }

    fn virt_test() {
        let prog = new Self();
        std::IO::writeln(prog.hi_count);        // 0
        prog.hi();                              // HI
        std::IO::writeln(prog.hi_count);        // 1
        prog.hi();                              // HI
        std::IO::writeln(prog.hi_count);        // 2

        let derived_ref = new Derived();
        derived_ref.say();      // 1001
        let base_ref = new Base();
        base_ref.say();         // 1000
        base_ref = derived_ref as Base;
        base_ref.say();         // 1001
    }

    fn str_arr_test() {
        // let arr = new int[100];
    }

    fn main() {
        Self::logic_test();
        Program::virt_test();
        Program::str_arr_test();
    }
}
