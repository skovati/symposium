# symposium
a refreshingly simple web chat client & server.

![screenshot](https://user-images.githubusercontent.com/49844593/149445309-c93e3d8a-1e01-4129-b59a-5e23c5effbae.png)

## features
- [X] web UI
- [X] encrypted WebSocket support w/ [warp](https://lib.rs/crates/warp)
- [X] asynchronous & concurrent w/ [tokio](https://tokio.rs/)
- [X] multi-user support
- [X] user authentication & authorization
- [X] login page

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
- [ ] commands (eg /list /join)
