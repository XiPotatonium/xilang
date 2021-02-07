/* UTF-8 is supported
 * Author: Xi
 */
// Compound Test

class Program {
	fn main() {
		let arrs : [i32;10];
		System::IO::Write("Hello world!\n");
		let d1 = new Demo(2, 12);
		let d: Demo = new Demo(1, 12);
		let x: i32 = Gcd(27, d.Foo(5));		// GCD(27, 18)
		System::IO::Write(x);
		System::IO::PutChar(10);	// \n
	}

	fn Gcd(a: i32, b: i32) -> i32 {
		if b == 0 {
			a
		} else {
			Gcd(b, a % b)
		}
	}
}

class Demo {
 	static Tag: i32;
	let Id: i32;
	let Value: i32;

	fn Demo(self, id: i32) {
		self.Id = id;
		self.Value = 20;
		Tag = Tag + 1;
	}

 	fn Foo(self, a: i32) -> i32 {
 		self.Value + a + self.Id
 	}
}
