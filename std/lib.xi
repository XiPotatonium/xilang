
// put into io.xi after pub use is available
class IO {
    #[Dllimport("xtd.dll")]
    fn putchar(ch: i32);

    #[Dllimport("xtd.dll")]
    fn puti32(i: i32);
}
