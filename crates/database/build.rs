fn main() {
    // Tell cargo to pass the SQLX_OFFLINE env var to rustc
    println!("cargo:rerun-if-env-changed=DATABASE_URL");
    println!("cargo:rerun-if-env-changed=SQLX_OFFLINE");
}
