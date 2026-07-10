# Checkpoint

1. Why does `run_echo` take `R: BufRead` instead of `R: Read`? What method
   does it need that `Read` alone doesn't give you?
2. The end-to-end test calls `client.shutdown(Shutdown::Write)` before
   reading the response. What would `run_echo`'s loop do if that line were
   deleted — would `read_line` ever return `Ok(0)`? Try deleting it and
   running the test to find out (put it back afterward).
3. `serve_once` calls `stream.try_clone()` to get two handles to the same
   connection instead of using one `&mut stream` for both reading and
   writing. `run_echo`'s signature is
   `run_echo<R: BufRead, W: Write>(reader: R, writer: W)` — two *separate*
   parameters. Given what you know about borrowing (one mutable borrow at a
   time), why can't both `R` and `W` be `&mut TcpStream` pointing at the
   same stream?
4. `main.rs` spawns one OS thread per connection. What's the actual resource
   cost of that as simultaneous clients grow into the thousands, and what
   does Rust's ecosystem offer instead (you'll start using it next module)?
5. In Python, `socket.create_connection(...).recv(...)` blocks the calling
   thread the same way Rust's `TcpStream::read` does. Given Python's GIL,
   what would happen if you tried to handle thousands of concurrent sockets
   with one OS thread per connection, the same way `main.rs` does here?
   (This is *exactly* why `asyncio`/ASGI exist in the Python world — name
   the Rust equivalent you'll meet next module.)
