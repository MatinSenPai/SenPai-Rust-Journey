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

خط جالب توجه این کد `for s in strings` هستش. نوعِ `strings` اینجا `Vec<String>` در نظر گرفته شده (یعنی دارای مالکیته نه قرض‌گرفته شده) و عبارتِ `for x in some_vec` (دقت کن نه `&some_vec`) تابع `IntoIterator::into_iter` رو به‌صورتِ "با ارزشِ خود مقدار (by value)" صدا می‌زنه — این کار، وکتور (vec) رو **مصرف می‌کنه (consumes)** و به ازای هر دور چرخش، یه `String` تحت مالکیت رو تو متغیر `s` تحویلت می‌ده. این دقیقاً همون قراردادیه که می‌گه "تابع total_length مالکیت کامل رو می‌خواد" و امضای تابع هم همینو قول داده: زمانی که این حلقه تموم بشه، تک‌تک `String`هایی که تو `strings` بودن یا شمرده شدن و بعد پاک شدن (در پایانِ هر تکرار حلقه، وقتی که اسکوپِ `s` تموم می‌شه)، و در نهایت خود `strings` هم کلاً از بین رفته.

در مورد سؤال اولِ چک‌پوینت: از کامنت خارج کردنِ `moved_value_demo` و اجرای `cargo check` یه همچین چیزی رو می‌ده:

```
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

خطای `E0382` به‌طور خاص یعنی «سعی کردی مقداری رو که قبلاً منتقل (move) شده، استفاده کنی».
پیغام خطا حتی بهت می‌گه *چرا* این انتقال رخ داده (`String` traitای به اسم `Copy` رو نداره) — این دقیقاً موضوع درس بعدی‌مونه.