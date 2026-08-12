# پاسخ تشریحی

در `sum_in_threads` هر thread یک chunk مستقل با مالکیت خودش می‌گیرد و فقط `i32` نهایی را از `JoinHandle` پس می‌دهد؛ هیچ مقدار تغییرپذیر مشترکی وجود ندارد. در شمارش مشترک، cloneهای `Arc` مالک همان `Mutex<i32>` هستند و هر افزایش داخل guard انجام می‌شود.

الگوی اصلی:

```rust
let counter = Arc::new(Mutex::new(0));
let handle_counter = Arc::clone(&counter);
let handle = std::thread::spawn(move || {
    *handle_counter.lock().unwrap() += 1;
});
handle.join().unwrap();
let final_count = *counter.lock().unwrap();
final_count
```

`Mutex` به‌تنهایی یک مالک دارد و move اول آن را از دسترس threadهای بعدی خارج می‌کند. `Arc` مالکیت مشترک فراهم می‌کند؛ `Mutex` به‌تنهایی مالکیت را تکثیر نمی‌کند. binding نهایی نیز guard temporary را پیش از dropشدن `counter` در انتهای بلوک مدیریت می‌کند و از تداخل drop order در tail expression جلوگیری می‌کند.
