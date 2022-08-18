
// println!("cargo:rustc-link-search=/opt/oracle/instantclient_19_3/");
// println!("cargo:rustc-link-lib=oci");
// println!("cargo:rustc-link-lib=clntsh");
// println!("cargo:rustc-link-lib=nnz19");
// println!("cargo:rustc-link-lib=mql1");
// println!("cargo:rustc-link-lib=ipc1");
// println!("cargo:rustc-link-lib=clntshcore");

use std::env;

#[cfg(target_os = "windows")]
const OCI_LIB: &str = "oci";

#[cfg(not(target_os = "windows"))]
const OCI_LIB: &str = "clntsh";

fn main() {
    if let Ok(lib_dir) = env::var("OCI_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", lib_dir);
    } else {
        panic!("Please set OCI_LIB_DIR to build oci-bindings");
    }
    println!("cargo:rustc-link-lib={}", OCI_LIB);
    println!("cargo:rerun-if-env-changed=OCI_LIB_DIR");
}