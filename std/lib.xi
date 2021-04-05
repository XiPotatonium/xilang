
// put into io.xi after pub use is available
class IO {
    #[Dllimport("std.dll")]
    fn putchar(ch: i32);
}
