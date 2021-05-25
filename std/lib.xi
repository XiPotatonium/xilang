
// put into io.xi after pub use is available
class IO {
    static SPACE: i32;
    static NEW_LINE: i32;

    static {
        SPACE = 32;
        NEW_LINE = 10;
    }

    #[Dllimport("xtd.dll")]
    fn putchar(ch: i32);

    #[Dllimport("xtd.dll")]
    fn puti32(i: i32);

    #[InternalCall]
    fn write(s: string);

    fn writeln(s: string) {
        Self::write(s);
        Self::putchar(Self::NEW_LINE);
    }

    fn write(i: i32) {
        Self::puti32(i);
    }

    fn writeln(i: i32) {
        Self::write(i);
        Self::putchar(Self::NEW_LINE);
    }
}

// Root of all classes
class Object {
    virtual fn repr() -> string {
        "std::Object"
    }
}

// Root of all value types
class ValueType {

}

struct I32 {
   let value: i32;
}


class String {
    #[InternalCall]
    fn len(self) -> i32;
}

class Array {
    // let len: usize;
}
