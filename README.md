# symposium
a refreshingly simple web chat client & server.

## features
- [X] web UI
- [X] encrypted WebSocket support w/ [warp](https://lib.rs/crates/warp)
- [X] asynchronous & concurrent w/ [tokio](https://tokio.rs/)
- [X] multi-user support
- [X] usernames

## usage
### server
```bash
git clone https://github.com/symposium
cd symposium
cargo run
```
### client
visit http://127.0.0.1:8080

## to-do
- [ ] direct messages
- [ ] topic channels
- [ ] message history
- [ ] user authentication/registration
