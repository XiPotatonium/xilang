/* UTF-8 is supported
 * Author: Xi
 */
// Compound Test
mod Demo;
mod Unit;

/*
class Program {
    fn main() {
        let arrs : [i32;] = new [i32;10];
        System::IO::puts("Hello world!\n");
        let d0 = Demo::new(2, 12);
        let d: Demo = Demo::new(1, 18);
        let x: i32 = gcd(27, d.value);
        System::IO::puti(x);
        System::IO::putc(10);	// \n
        System::IO::puti(d0.foo(5));
        System::IO::putc(10);	// \n
    }

    fn gcd(a: i32, b: i32) -> i32 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }
}

class Demo {
    static TAG: i32;
    let id: i32;
    let value: i32;

    static {
        Demo::TAG = 0;
    }

    fn new(id: i32, value: i32) -> Demo {
        let ret = new Demo {
            id,
            value: value,
        };
        Demo::TAG = Demo::TAG + 1;
        ret
    }

    fn foo(self, a: i32) -> i32 {
        self.value + Demo::TAG + a + self.id
    }
}
*/
