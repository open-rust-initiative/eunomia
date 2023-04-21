use std::thread;

static mut UNSAFE: i32 = 0;

fn main() {
  let t1 = thread::spawn(|| unsafe { UNSAFE = 1 });
  let t2 = thread::spawn(|| unsafe { UNSAFE = 2 });
  t1.join().unwrap();
  t2.join().unwrap();
}
