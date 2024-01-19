use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    if env::var("DOCS_RS").is_ok() && env::var("RUSTIC_USER_PAGE_END").is_err() {
        env::set_var("RUSTIC_USER_PAGE_END", "1024");
    }
    if env::var("DEBUG").is_ok() && env::var("RUSTIC_USER_PAGE_END").is_err() {
        env::set_var("RUSTIC_USER_PAGE_END", "1024");
    }

    #[allow(clippy::expect_used)] // safe unwrap during build
        let user_page_end = env::var("RUSTIC_USER_PAGE_END").expect("Please set environment variable RUSTIC_USER_PAGE_END, such as `export RUSTIC_USER_PAGE_END=1024`. If the canister is already deployed, query `get_config_user_page_end()` for the correct value.");
    assert!(user_page_end.parse::<u64>().is_ok());

    #[allow(clippy::unwrap_used)] // safe unwrap during build
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("config.rs");
    #[allow(clippy::unwrap_used)] // safe unwrap during build
    let mut f = File::create(dest_path).unwrap();

    #[allow(clippy::unwrap_used)] // safe unwrap during build
    f.write_all(format!("pub const USER_PAGE_END: u64 = {};\n", user_page_end).as_bytes())
        .unwrap();
}
