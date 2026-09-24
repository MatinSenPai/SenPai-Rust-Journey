# Solution — 3.1.1 TCP echo server

```rust
pub fn run_echo<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<usize> {
    let mut total = 0usize;
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }
        writer.write_all(line.as_bytes())?;
        writer.flush()?;
        total += bytes_read;
    }

    Ok(total)
}
```

The loop clears `line` at the top of every iteration rather than declaring a
fresh `String` each time — `read_line` *appends* to whatever's already in
the buffer, it doesn't overwrite it, so a stale `line` from the previous
iteration would silently accumulate every line the client ever sent. That's
a real, easy-to-make bug: the first test run would look correct (`line` was
empty on iteration one) and only break once more than one line came through.

`bytes_read == 0` is the EOF signal `read_line` gives you — not an empty
line (`"\n"`, one byte) but genuinely nothing left to read, which happens
once the other side closes its write half of the connection. Any other
positive count, even for a line with no trailing `\n` (the stream just
ended mid-line), means "here's more data," so it gets written straight
back. `examples/03-missing-eof-check-broken.rs` is what happens when this
check is missing entirely: `read_line` keeps returning `Ok(0)` forever
without ever blocking, so the loop spins instead of breaking.

```rust
pub fn serve_once(listener: &TcpListener) -> io::Result<usize> {
    let (stream, _addr) = listener.accept()?;
    let writer: TcpStream = stream.try_clone()?;
    let reader = io::BufReader::new(stream);
    run_echo(reader, writer)
}
```

`try_clone` doesn't duplicate the connection — it duplicates the *handle* to
the same underlying OS socket (internally, it's a `dup()` of the file
descriptor). Both `stream` and `writer` refer to the same conversation with
the client; reading from one and writing to the other work exactly like
reading/writing the original single stream would. This sidesteps a real
borrow-checker wall: `run_echo`'s signature takes `reader: R` and
`writer: W` as two independent parameters, and the borrow checker will
never let you hand it `&mut stream` twice — that's exactly the `E0499` in
`../README.md`'s "Errors you will meet". Two cloned handles are two
genuinely separate, owned values, so each moves into `run_echo`
independently, no borrow conflict at all.

## On thread-per-connection cost

Every `std::thread::spawn` call reserves a real OS thread — typically a
multi-megabyte stack, plus one more thing for the kernel scheduler to
context-switch between. Thousands of *idle* connections (a chat app with
quiet users, a webhook receiver waiting on rare events) means thousands of
threads sitting there doing nothing but costing memory and scheduler
overhead. `tokio` (already met in Phase 2, and applied for real starting
module 2) solves this by multiplexing many *lightweight, cooperatively
scheduled* tasks onto a small, fixed pool of OS threads — a task that's just
waiting on I/O doesn't hold a thread hostage, it hands the thread back to
the runtime, which gives it to whichever task actually has work ready to
do. Same fundamental idea as Python's `asyncio`, just without a GIL and
compiled instead of interpreted.
