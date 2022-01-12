# symposium
a very simple tcp text chat server compatible with netcat and telnet.

## features
- [X] asynchronous & concurrent w/ [tokio](https://tokio.rs/)
- [X] multi-user support
- [X] compatability with any UTF-8/TCP client
- [X] usernames

## usage
### server
```bash
git clone https://github.com/symposium
cd symposium
cargo run
```
### client
```bash
nc 127.0.0.1 8080
> enter username: skovati
> [skovati]: hello world!
```

## to-do
- [ ] custom client
- [ ] direct messages
- [ ] message history
- [ ] user authentication/registratoin
- [ ] web UI
- [ ] SSL/TLS
