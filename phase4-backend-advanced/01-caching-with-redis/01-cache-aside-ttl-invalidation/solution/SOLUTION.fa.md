# راه‌حل

```rust
pub fn cache_key(namespace: &str, id: &str) -> String {
    format!("{namespace}:{id}")
}
```

ارزش تابع در یکدست‌کردن discipline است: key را در هر call site دستی نساز تا namespaceها تصادفی collision نکنند.

```rust
async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
    match self.store.get(key) {
        Some((value, expires_at)) if self.clock.now() < *expires_at => Ok(Some(value.clone())),
        _ => Ok(None),
    }
}

async fn set(&mut self, key: &str, value: String, ttl: Duration) -> Result<(), CacheError> {
    self.store.insert(key.to_string(), (value, self.clock.now() + ttl));
    Ok(())
}

async fn invalidate(&mut self, key: &str) -> Result<(), CacheError> {
    self.store.remove(key);
    Ok(())
}
```

guardِ `self.clock.now() < *expires_at` کل مکانیزم TTL است: نبودن entry و entry منقضی برای `get_or_set` هر دو miss هستند. `set` عمداً key قبلی را overwrite می‌کند.

```rust
pub async fn get_or_set<C, F, Fut>(
    cache: &mut C,
    key: &str,
    ttl: Duration,
    compute: F,
) -> Result<String, CacheError>
where
    C: Cache,
    F: FnOnce() -> Fut,
    Fut: Future<Output = String>,
{
    if let Some(value) = cache.get(key).await? {
        return Ok(value);
    }
    let value = compute().await;
    cache.set(key, value.clone(), ttl).await?;
    Ok(value)
}
```

`FnOnce() -> Fut` باعث می‌شود compute فقط در miss اجرا شود؛ API eager هزینهٔ گران را حتی در hit می‌پردازد. expired entry را با `&self` پاک نمی‌کنیم، پس readها exclusive borrow نمی‌خواهند؛ cache واقعی برای کنترل memory می‌تواند sweep پس‌زمینه داشته باشد. برای stampede باید برای key در حال محاسبه lock/single-flight داشته باشی تا callerهای بعدی منتظر نتیجهٔ اول بمانند. `Cell<Instant>` هم interior mutability سبک برای clock تک‌thread است؛ بدون `Mutex`، `advance(&self)` را ممکن می‌کند.
