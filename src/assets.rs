#[cfg(host_windows)]
macro_rules! host_path_separator {
    () => {
        r"\"
    };
}

#[cfg(not(host_windows))]
macro_rules! host_path_separator {
    () => {
        r"/"
    };
}

pub mod fonts;
