struct String {
    #[internal]
    fn len(self);
}

#[internal]
fn prints(str: String);
#[internal]
fn println(str: String);
#[internal]
fn printc(ch: char);
#[internal]
fn printi(v: i32);
