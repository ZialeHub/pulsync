#[cfg(not(any(feature = "sync", feature = "async")))]
compile_error!("You must enable one of ['sync', 'async'] features");
#[cfg(all(feature = "sync", feature = "async"))]
compile_error!("You must enable either 'sync' or 'async' feature, not both");
fn main() {}
