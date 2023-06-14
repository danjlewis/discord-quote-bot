fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rust-cfg=host_windows");
        println!("cargo:rust-cfg=host_family=windows");
    }

    #[cfg(unix)]
    {
        println!("cargo:rust-cfg=host_unix");
        println!("cargo:rust-cfg=host_family=unix");
    }
}
