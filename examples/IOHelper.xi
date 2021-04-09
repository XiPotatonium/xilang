
class HelloWorld {
    fn hi() {
        std::IO::putchar(72);
        std::IO::putchar(73);
        std::IO::putchar(10);
    }
}

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
