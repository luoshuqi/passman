A simple password manager written in Rust.

### Usage

build: 

```bash
cd html && yarn build && cd ../ && cargo build --release
```

run:

```bash
target/release/passman --bind 127.0.0.1:8888 --data-dir . --allow-create-user
```

visit `http://127.0.0.1:8888/user/create` to create user