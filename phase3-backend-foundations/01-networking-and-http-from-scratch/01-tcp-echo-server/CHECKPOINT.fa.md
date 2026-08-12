# ایست بازرسی

۱. چرا `run_echo` به‌جای `R: Read`، `R: BufRead` می‌گیرد؟ به کدام متد نیاز دارد که `Read` به‌تنهایی ندارد؟

۲. تست end-to-end پیش از خواندن پاسخ `client.shutdown(Shutdown::Write)` را صدا می‌زند. اگر آن خط حذف شود، حلقه‌ی `run_echo` چه می‌کند؟ آیا `read_line` هرگز `Ok(0)` می‌دهد؟

۳. چرا `serve_once` با `stream.try_clone()` دو handle از یک اتصال می‌سازد، به‌جای یک `&mut stream` برای خواندن و نوشتن؟ با قانون «یک borrow تغییرپذیر در هر لحظه» توضیح بده چرا هر دو `R` و `W` نمی‌توانند `&mut TcpStream` یکسان باشند.

۴. یک OS thread برای هر اتصال، با هزاران client هم‌زمان دقیقاً چه هزینه‌ای دارد؟ اکوسیستم Rust چه جایگزینی می‌دهد؟

۵. `socket.create_connection(...).recv(...)` در پایتون مانند `TcpStream::read` block می‌شود. با GIL، الگوی هزار thread برای هزار socket چه محدودیتی دارد؟ همتای Rust ابزار `asyncio`/ASGI که ماژول بعد می‌بینی چیست؟
