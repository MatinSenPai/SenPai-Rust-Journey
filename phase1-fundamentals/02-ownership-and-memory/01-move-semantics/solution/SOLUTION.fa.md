# راه‌حل

```rust
pub fn total_length(strings: Vec<String>) -> usize {
    let mut total = 0;
    for s in strings {
        total += s.len();
    }
    total
}
```

خط مهم `for s in strings` است. چون `strings` از نوع دارای مالکیت `Vec<String>` است، حلقه `IntoIterator::into_iter` را با مقدار فراخوانی و بردار را مصرف می‌کند. در هر تکرار، مالکیت یک `String` به `s` می‌رسد؛ پس از شمارش byteها، با پایان محدوده همان تکرار آزاد می‌شود. در پایان، خود `strings` نیز دیگر وجود ندارد.

برای پرسش اول، خروجی تقریباً چنین است:

```text
error[E0382]: borrow of moved value: `s`
 --> src/lib.rs:XX
  |
  | let s = String::from("hello");
  | - move occurs because `s` has type `String`, which does not implement the `Copy` trait
  | let s2 = s;
  |          - value moved here
  | println!("{s}");
  |           ^ value borrowed here after move
```

`E0382` می‌گوید بعد از انتقال از مقدار استفاده کرده‌ای. خود پیام سه سرنخ می‌دهد: نوع برابر `String` است، این نوع صفت `Copy` ندارد و انتقال در `let s2 = s` رخ داده است.

نکته‌ی UTF-8: متد `String::len()` تعداد byteها را می‌دهد، نه تعداد characterها. برای `"سلام"` مقدار `len()` برابر ۸ است، چون هر حرف فارسی در UTF-8 دو byte دارد. این تمرین عمداً طول byte را جمع می‌کند.
