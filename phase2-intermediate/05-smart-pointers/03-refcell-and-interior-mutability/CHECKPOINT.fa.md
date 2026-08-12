# ایست بازرسی

۱. تست `a_second_borrow_mut_panics_while_the_first_is_still_alive` را اجرا و panic را بخوان. کدام invariant مشخص `RefCell` نقض شده است؟

۲. `increment` و `get` هر دو `&self` می‌گیرند. اگر `SharedCounter` فقط `inner: Rc<i32>` داشت، آیا می‌توانستی `increment(&self)` واقعاً تغییر‌دهنده بنویسی؟ چرا؟

۳. `SharedCounter`، `Clone` را derive می‌کند. هنگام `.clone()` یک `i32` تازه ساخته می‌شود یا چیز دیگری clone می‌شود؟ این پاسخ موفقیت تست `clones_share_the_same_underlying_counter` را چگونه توضیح می‌دهد؟

۴. یک ساختمان داده یا سناریوی دیگر نام ببر که `Rc<RefCell<T>>` لازم دارد و توضیح بده چرا مالک یکتا همراه `&mut` مناسب نیست.
