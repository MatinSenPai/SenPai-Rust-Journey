# پاسخ تشریحی

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

در ابتدای هر دور `line` را پاک می‌کنیم، نه اینکه هر بار `String` تازه بسازیم. `read_line` به buffer موجود **اضافه** می‌کند و آن را overwrite نمی‌کند؛ پس بدون `clear` داده‌ی خط قبل بی‌صدا با خط تازه جمع می‌شد. این باگ در نخستین خط دیده نمی‌شود و فقط با ورود چند خط آشکار می‌شود.

`bytes_read == 0` علامت EOF است، نه خط خالی. یک خط خالی `"\n"` هنوز یک byte دارد. EOF وقتی می‌آید که طرف دیگر نیمه‌ی نوشتن اتصال را ببندد. هر شمارش مثبت، حتی خط آخر بدون `\n`، داده‌ی قابل echo است.

```rust
pub fn serve_once(listener: &TcpListener) -> io::Result<usize> {
    let (stream, _addr) = listener.accept()?;
    let writer: TcpStream = stream.try_clone()?;
    let reader = io::BufReader::new(stream);
    run_echo(reader, writer)
}
```

`try_clone` اتصال را duplicate نمی‌کند؛ handle مربوط به همان socket سیستم‌عامل را کپی می‌کند. `stream` و `writer` یک گفت‌وگوی client را می‌بینند. این کار مانع borrow conflict می‌شود: `run_echo` دو پارامتر مستقل می‌گیرد و Rust هرگز اجازه نمی‌دهد یک `&mut stream` را دوبار هم‌زمان به آن بدهی. دو handle cloneشده دو value مستقل‌اند و هر دو می‌توانند move شوند.

هر `std::thread::spawn` یک thread واقعی با stack و هزینه‌ی context switching می‌خواهد. هزاران اتصال idle یعنی هزاران thread بی‌کار اما پرهزینه. `tokio` taskهای سبک و cooperative را روی pool کوچکی از OS threadها multiplex می‌کند؛ task منتظر I/O thread را آزاد می‌کند. این همان ایده‌ی بنیادین `asyncio` در پایتون است، اما بدون GIL و با کد compileشده.
