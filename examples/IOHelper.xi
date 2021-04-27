class Helper {
    static SPACE: i32;
    static NEW_LINE: i32;

    static {
        SPACE = 32;
        NEW_LINE = 10;
    }

    fn write_i32_ln(v: i32) {
        std::IO::puti32(v);
        std::IO::putchar(Self::NEW_LINE);
    }
}

class IOBase {
    let hi_count: i32;

    fn hi(self) {
        std::IO::putchar(72);
        std::IO::putchar(73);
        std::IO::putchar(Helper::NEW_LINE);

        self.hi_count = self.hi_count + 1;
    }
}
