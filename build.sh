cd client
cargo build --release
cd ../servers
cargo build --release --bin server1
cargo build --release --bin server2
cd ../logserver
cargo build --release
cd ..
