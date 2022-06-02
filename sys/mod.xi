struct String {
    #[internal]
    fn len(self);
}

enum opt<T> {
    Some(T),
    None(),
}

#[internal]
fn prints(s: str);
#[internal]
fn println(s: str);
#[internal]
fn printc(ch: char);
#[internal]
fn printi(v: i32);
