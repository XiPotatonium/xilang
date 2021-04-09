mod algo;

class Demo {
    static TAG: i32;
    let id: i32;
    let value: i32;

    static {
        let foo: i32 = 0;
        Demo::TAG = foo;
    }

    fn create(id: i32, value: i32) -> Self {
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