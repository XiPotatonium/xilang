class Helper {}

class IOBase {
    let hi_count: i32;

    fn hi(self) {
        std::IO::putchar(72);
        std::IO::putchar(73);
        std::IO::putchar(std::IO::NEW_LINE);

        self.hi_count = self.hi_count + 1;
    }
}
