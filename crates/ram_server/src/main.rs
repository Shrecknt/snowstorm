fn main() {
    println!("Hello, World!");
    let mut handle =
        ram_server::run_server("1.20.4-paper", 25565, true).expect("Could not start server :<");
    println!("server started!");
    handle.kill().expect("Unable to kill server process");
}
