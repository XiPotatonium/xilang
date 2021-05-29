/* UTF-8 is supported
 * Author: Xi
 */
// Compound Test
mod demo;
mod IOHelper;

class Base {
    virtual fn say(self) {
        std::IO::writeln("Say from Base class");
    }
}

class Derived: Base {
    override fn say(self) {
        std::IO::writeln("Say from Derived class");
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
        std::IO::writeln("String&Array Test:");
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
        std::IO::writeln("Basic Logic Test:");
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
        std::IO::writeln("Virtual Method Test:");
        Self::singleton = new Self();
        std::IO::writeln(Self::singleton.hi_count);         // 0
        Self::singleton.hi();                               // HI
        std::IO::writeln(Self::singleton.hi_count);         // 1
        Self::singleton.hi();                               // HI
        std::IO::writeln(Self::singleton.hi_count);         // 2

        let derived_ref = new Derived();
        derived_ref.say();      // derived
        let base_ref = new Base();
        base_ref.say();         // base
        base_ref = derived_ref as Base;
        base_ref.say();         // derived
    }

    fn main() {
        Self::logic_test();
        Self::virt_test();
        Self::str_arr_test();
    }
}
