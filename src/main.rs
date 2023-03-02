fn main() {
    let path = env::args().nth(1).unwrap_or({
        eprintln!("Path is not provided, serving current directory");
        String::from(".")
    });
}
