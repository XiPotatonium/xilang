mod algo;

class Demo {
    static TAG: i32;
    let id: i32;
    let value: i32;

    static {
        let foo: i32 = 0;
        Demo::TAG = foo;
    }

    Self(self, id: i32, value: i32) {
        self.id = id;
        self.value = value;
        Self::TAG = Self::TAG + 1;
    }

    Self(self, id: i32) {
        self.id = id;
        self.value = 0;
        Demo::TAG = Demo::TAG + 1;
    }

    fn foo(self, a: i32) -> i32 {
        self.value + Demo::TAG + a + self.id
    }
}