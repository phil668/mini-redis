use atoi::atoi;
fn main() {
    let str = String::from("1234556hello");
    let u8s = str.as_bytes();
    println!("{:?}", atoi::<u64>(u8s));
}
