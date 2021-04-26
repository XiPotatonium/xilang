
// put into io.xi after pub use is available
class IO {
    #[Dllimport("std.dll")]
    fn putchar(ch: i32);

    #[Dllimport("std.dll")]
    fn puti32(i: i32);
}

// Root of all classes
class Object {}
