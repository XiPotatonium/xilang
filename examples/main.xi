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

class Int32 {
    let val: i32;

    Self(self, v: i32) {
        self.val = v;
    }
}

class Program: IOHelper::IOBase {

    fn str_arr_test() {
        let s: string = "Hello world!";
        std::IO::write(s.len());
        std::IO::writeln(s);
        let arr: Int32[] = new Int32[10];
        let i = 0;
        loop {
            if i >= arr.len {
                break;
            }
            arr[i] = new Int32(i);
            i = i + 1;
        }
        loop {
            i = i - 1;
            if i < 0 {
                break;
            }
            std::IO::write(arr[i].val);
            std::IO::putchar(std::IO::SPACE);
        }
        std::IO::putchar(std::IO::NEW_LINE);
    }
    
    fn logic_test() {
        let d: demo::Demo = new crate::demo::Demo(1, 24);
        let a = d.foo(6);               // 32
        std::IO::writeln(a);
        let d: i32 = demo::algo::Algorithm::gcd(a, d.value);    // gcd(32, 24)
        std::IO::writeln(d);
        let d = new demo::Demo(30);
        std::IO::writeln(d.foo(4));     // 0 + 2 + 4 + 30
    }

    static singleton: Program;

    fn virt_test() {
        Self::singleton = new Self();
        std::IO::writeln(Self::singleton.hi_count);         // 0
        Program::singleton.hi();                            // HI
        std::IO::writeln(Program::singleton.hi_count);      // 1
        Self::singleton.hi();                               // HI
        std::IO::writeln(Self::singleton.hi_count);         // 2

        let derived_ref = new Derived();
        derived_ref.say();      // 1001
        let base_ref = new Base();
        base_ref.say();         // 1000
        base_ref = derived_ref as Base;
        base_ref.say();         // 1001
    }

    fn main() {
        Self::logic_test();
        Program::virt_test();
        Program::str_arr_test();
    }
}
