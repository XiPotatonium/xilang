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

struct MyPair {
    static DEFAULT: MyPair;

    let key: i32;
    let val: string;

    Self(self, key: i32, val: string) {
        self.key = key;
        self.val = val;
    }

    fn print(self) {
        std::IO::write("{ ");
        std::IO::write(self.key);
        std::IO::write(": \"");
        std::IO::write(self.val);
        std::IO::write("\" }");
    }
}

class Program: IOHelper::IOBase {

    fn str_test() {
        std::IO::writeln("String Test:");
        let s: string = "Hello world!";
        std::IO::write(s.len());
        std::IO::writeln(s);
    }

    fn arr_test() {
        std::IO::writeln("Array Test:");
        let arr: i32[] = new i32[10];
        let i = 0;
        loop {
            if i >= arr.len {
                break;
            }
            arr[i] = i;
            i = i + 1;
        }
        loop {
            i = i - 1;
            if i < 0 {
                break;
            }
            std::IO::write(arr[i]);
            std::IO::putchar(std::IO::SPACE);
        }
        std::IO::putchar(std::IO::NEW_LINE);

        let struct_arr = new MyPair[3]; 
        i = 0;
        loop {
            if i >= struct_arr.len {
                break;
            }
            struct_arr[i] = new MyPair(i, "MyPairs");
            struct_arr[i].val = "MyPair";
            i = i + 1;
        }
        loop {
            i = i - 1;
            if i < 0 {
                break;
            }
            struct_arr[i].print();
            std::IO::putchar(std::IO::SPACE);
        }
        std::IO::putchar(std::IO::NEW_LINE);
    }

    fn value_type_test() {
        std::IO::writeln("Value type Test:");
        // value array is tested at str_arr_test()
        let pair: MyPair = new MyPair(101, "This is my pair");
        std::IO::writeln(pair.key);
        std::IO::writeln(pair.val);
        pair.key = 102;
        pair.val = "This is also my pair";
        pair.print();

        MyPair::DEFAULT = new MyPair(102, "MyPair::DEFAULT");
        MyPair::DEFAULT.print();
        std::IO::writeln(MyPair::DEFAULT.val);
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
        Self::str_test();
        Self::arr_test();
        Self::value_type_test();
    }
}
