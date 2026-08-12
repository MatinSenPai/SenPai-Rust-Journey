# راه‌حل

```rust
pub fn new(capacity: u32, refill_rate: f64, now: Instant) -> Self {
    Self {
        capacity: capacity as f64,
        refill_rate,
        tokens: capacity as f64,
        last_refill: now,
    }
}

fn refill(&mut self, now: Instant) {
    let elapsed = now.saturating_duration_since(self.last_refill).as_secs_f64();
    self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
    self.last_refill = now;
}

pub fn try_acquire(&mut self, now: Instant) -> bool {
    self.refill(now);
    if self.tokens >= 1.0 {
        self.tokens -= 1.0;
        true
    } else {
        false
    }
}
```

شروعِ پر، burst اولیه تا `capacity` را مجاز می‌کند. `saturating_duration_since` زمان عقب‌رفته را دفاعی صفر می‌گیرد و `.min(self.capacity)` هرگز overflow نمی‌گذارد. refill باید پیش از تصمیم باشد؛ requestی که دقیقاً یک token پس از idle به دست آورده نباید اشتباه رد شود.

`f64` creditهای کسری را نگه می‌دارد؛ با integer، refillهای ۰٫۲تایی مدام truncate می‌شوند مگر fixed-point اضافی بسازی. `last_refill` «آخرین زمان حساب‌شده» است، نه آخرین زمانی که token کامل اضافه شد؛ هر call باید بازهٔ زمانی را دقیقاً یک‌بار حساب کند.

TTL می‌پرسد داده هنوز قابل‌اعتماد است یا stale؛ token bucket می‌پرسد آیا اخیراً کار زیادی انجام شده و credit تا چه اندازه بازیابی شده. `try_acquire` رد فوری است. نسخهٔ async باید زمان token بعدی را حساب و `sleep`/`Notify` کند، fairness و queue محدود را هم صریح طراحی کند؛ صرف اضافه‌کردن `.await` آن‌ها را خلق نمی‌کند.
