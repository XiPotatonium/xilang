/* UTF-8 is supported
 * Author: Xi
 */
// Basic unit test
mod Loop;
mod Branch;

class Program {
    fn main() {
        let d = Demo::new(1, 100);
        d.foo(16);
    }
}

class Demo {
    static TAG: i32;
    let id: i32;
    let value: i32;

    static {
        let foo: i32 = 0;
        Demo::TAG = foo;
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
