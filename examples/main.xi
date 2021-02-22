/* UTF-8 is supported
 * Author: Xi
 */
// Compound Test
mod demo;

class Program {
    fn main() {
        let d = crate::demo::Demo::new(1, 100);
        d.foo(16);
        let d: i32 = d.value;
    }
}
