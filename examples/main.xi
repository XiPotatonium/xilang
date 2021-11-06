/* UTF-8 is supported
 * Author: shwu
 */

struct Algorithm {
    fn gcd(a: i32, b: i32) -> i32 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }

    fn is_prime(n: i32) -> bool {
        if n == 1 {
            return false;
        }

        let i = 2;
        loop {
            if i * i > n {
                break;
            }

            if n % i == 0 {
                return false;
            }

            i = i + 1;
        }

        true
    }
}

struct Demo {
    static TAG: i32;
    let id: i32;
    let value: i32;

    fn foo(self, a: i32) -> i32 {
        self.value + Demo::TAG + a + self.id
    }
}

struct MyPair {
    let key: i32;
    let val: string;

    fn create(key: i32, value: string) -> MyPair {
        new Self { key, val: value }
    }
}

struct Program {

    static singleton: Program;

    fn str_test() {
        System::out.println("String Test:");
        let s: string = "Hello world!";
        System::out.print(s.len());
        System::out.println(s);
    }

    fn arr_test() {
        System::out.println("Array Test:");

        let struct_arr = new MyPair[3]; 
        let i = 0;
        loop {
            if i >= struct_arr.len {
                break;
            }
            struct_arr[i] = MyPair::create(i, "MyPairs");
            struct_arr[i].val = "MyPair";
            i = i + 1;
        }
        loop {
            i = i - 1;
            if i < 0 {
                break;
            }
            struct_arr[i].print();
            System::out.print(' ');
        }
        System::out.print('\n');

        let matrix: i32[][] = new i32[5][];
        i = 0;
        loop {
            if i >= matirx.len {
                break;
            }
            let row = new i32[3];
            let j = 0;
            loop {
                if j >= row.len {
                    break;
                }
                row[j] = i * row.len + j;
                j = j + 1;
            }
            matirx[i] = row;
            i = i + 1;
        }
        i = 0;
        loop {
            if i >= matirx.len {
                break;
            }
            let j = 0;
            loop {
                if j >= matirx[i].len {
                    break;
                }
                System::out.print(matrix[i][j]);
                System::out.print(' ');
                j = j + 1;
            }
            System::out.print('\n');
            i = i + 1;
        }
    }
    
    fn logic_test() {
        System::out.println("Basic Logic Test:");
        let d: Demo = new Demo { key: 1, value: 24 };
        let a = d.foo(6);               // 32
        System::out.println(a);
        let d: i32 = Algorithm::gcd(a, d.value);    // gcd(32, 24)
        System::out.println(d);
    }

    fn main() {
        Self::logic_test();
        Self::str_test();
        Self::arr_test();
    }
}