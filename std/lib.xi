
// put into io.xi after pub use is available
class IO {
    static SPACE: i32;
    static NEW_LINE: i32;

    static {
        SPACE = 32;
        NEW_LINE = 10;
    }

    #[Dllimport("std.dll")]
    fn putchar(ch: i32);

    #[Dllimport("std.dll")]
    fn puti32(i: i32);

    fn write(i: i32) {
        Self::puti32(i);
    }

    fn writeln(i: i32) {
        Self::write(i);
        Self::putchar(Self::NEW_LINE);
    }
}

// Root of all classes
class Object {}
