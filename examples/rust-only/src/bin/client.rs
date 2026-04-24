//! Rust-only example — client side.
//!
//! Run (server must be running first):
//!   cargo run --bin client
//!
//! Override the server address:
//!   BASE_URL=http://192.168.1.10:3000/transfer cargo run --bin client

use eruspy::client::EruspyClient;
use std::fs;

fn main() {
    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| "http://localhost:3000/transfer".to_owned());

    println!("[ rust client ] connecting to {base_url}");

    let c = EruspyClient::new(&base_url);

    // Upload a file
    fs::write("./hello.txt", "Hello from Rust client!\n").unwrap();
    c.upload("./hello.txt", "hello.txt").unwrap();
    println!("  uploaded   hello.txt");

    // List
    let entries = c.list("").unwrap();
    println!("  list root  ({} entries)", entries.len());
    for e in &entries {
        println!("    {} {}", if e.is_dir { "DIR " } else { "FILE" }, e.name);
    }

    // Download back
    c.download("hello.txt", "./received.txt").unwrap();
    println!("  downloaded hello.txt → received.txt");
    println!("  content: {:?}", fs::read_to_string("./received.txt").unwrap().trim());

    // Upload a directory
    fs::create_dir_all("./mydir").unwrap();
    fs::write("./mydir/a.txt", "alpha").unwrap();
    fs::write("./mydir/b.txt", "beta").unwrap();
    c.upload_dir("./mydir", "mydir").unwrap();
    println!("  uploaded   mydir/");

    // Download directory back
    let _ = fs::remove_dir_all("./mydir_restored");
    c.download_dir("mydir", "./mydir_restored").unwrap();
    println!("  downloaded mydir/ → mydir_restored/");

    // Cleanup
    let _ = fs::remove_file("./hello.txt");
    let _ = fs::remove_file("./received.txt");
    let _ = fs::remove_dir_all("./mydir");
    let _ = fs::remove_dir_all("./mydir_restored");

    println!("done.");
}
