# راه‌حل — ۳.۴.۱ پیکربندیِ ۱۲فاکتوری و سکرت‌ها

## `parse_bool`

```rust
if raw.eq_ignore_ascii_case("true") || raw == "1" {
    Ok(true)
} else if raw.eq_ignore_ascii_case("false") || raw == "0" {
    Ok(false)
} else {
    Err(ConfigError::Invalid { key, value: raw.to_string(), expected: "true, false, 1 or 0" })
}
```

`eq_ignore_ascii_case` حالت‌هایی مثلِ `TRUE` و `False` را بدونِ ساختنِ یک کپیِ حروفِ کوچک پوشش می‌دهد. عمداً چیزی trim نمی‌شود: `" true"` خطاست، چون یک فاصله‌ی اضافه در متغیرِ محیطی یک غلطِ املایی است که ارزشِ گزارش‌شدن دارد، نه پنهان‌شدن.

## `parse_origins`

```rust
raw.split(',')
    .map(str::trim)
    .filter(|piece| !piece.is_empty())
    .map(String::from)
    .collect()
```

`"a, b,,c "` می‌شود `["a", "b", "c"]`، و `""` یک `Vec`ِ خالی می‌دهد چون تنها تکه‌ی خالی فیلتر می‌شود.

## `Layer::from_env`

```rust
let port = match env.get("APP_PORT") {
    Some(raw) => Some(raw.parse::<u16>().map_err(|_| ConfigError::Invalid {
        key: "APP_PORT",
        value: raw.clone(),
        expected: "a port number from 0 to 65535",
    })?),
    None => None,
};
```

سه متغیرِ دیگر همین شکل را دارند: `APP_DEBUG` از `parse_bool` می‌گذرد، `APP_ALLOWED_ORIGINS` از `parse_origins`، و `APP_DB_PASSWORD` همان‌طور که هست با `SecretString::from` پیچیده می‌شود (trim نمی‌شود: فاصله می‌تواند بخشی از یک رمز باشد). `?`ِ داخلِ هر بازوی `match` در اولین متغیرِ بد، به ترتیبِ جدول، برمی‌گردد. `env.get` برایِ متغیرِ حاضر ولی خالی `Some("")` می‌دهد، پس `APP_PORT=` پارس نمی‌شود (رشته‌ی خالی یک `u16` نیست) و `APP_ALLOWED_ORIGINS=` می‌شود `Some(vec![])`، هر دو همان‌طور که کامنتِ مستندات می‌گوید. `"-1"`، `"70000"` و `"80 "` همگی در `u16::from_str` شکست می‌خورند.

## `Layer::over`

```rust
Layer {
    port: self.port.or(lower.port),
    allowed_origins: self.allowed_origins.or(lower.allowed_origins),
    debug: self.debug.or(lower.debug),
    db_password: self.db_password.or(lower.db_password),
}
```

`Option::or` کلِ قاعده‌ی اولویت است: اولین `Some` در هر فیلد برنده می‌شود. `or` در هر فیلد، به‌جایِ «کلِ لایه‌ی بالاتر را بردار اگر چیزی دارد»، همان چیزی است که اجازه می‌دهد محیط فقط همان یک تنظیمی را که نام می‌برد بازنویسی کند و بقیه را به فایل بسپارد.

## `Layer::finish`

```rust
Ok(Config {
    port: self.port.unwrap_or(DEFAULT_PORT),
    allowed_origins: self.allowed_origins.unwrap_or_else(|| vec![DEFAULT_ORIGIN.to_string()]),
    debug: self.debug.unwrap_or(false),
    db_password: self.db_password.ok_or(ConfigError::Missing { key: "db_password" })?,
})
```

برایِ مبدأها `unwrap_or_else`، چون ساختنِ `Vec` حافظه می‌گیرد و فقط وقتی لازم است باید اجرا شود. رمز تنها موردی است که `ok_or` دارد: پیش‌فرض ندارد، پس نبودنش خطاست.

## `Config::load`

```rust
let file_layer = match file {
    Some(text) => Layer::from_toml(text)?,
    None => Layer::default(),
};
Layer::from_env(env)?.over(file_layer).finish()
```

فایل اول پارس می‌شود، پس خطایِ فایل پیش از خطایِ محیط گزارش می‌شود، همان‌طور که کامنتِ مستندات قول می‌دهد. نبودنِ فایل فقط یک لایه‌ی خالی است، نه خطا. خواندنِ `config.toml` از دیسک عمداً این‌جا نیست: I/O را `main` انجام می‌دهد، تا `load` یک تابعِ خالص بماند که تست‌ها با رشته صدایش بزنند.

## درباره‌ی چالش

```rust
if let Some(origin) = origins.iter().find(|o| o.ends_with('/')) {
    return Err(ConfigError::Invalid {
        key: "APP_ALLOWED_ORIGINS",
        value: origin.clone(),
        expected: "an origin without a trailing slash",
    });
}
```

آن را در `finish` بگذار: تنها جایی است که هر لایه، فایل یا محیط، قبلاً ادغام شده، پس بررسی یک‌بار روی فهرستِ نهایی اجرا می‌شود. هزینه‌اش این است که خطا `APP_ALLOWED_ORIGINS` را نام می‌برد حتی وقتی مقدارِ بد از `allowed_origins`ِ فایل آمده باشد. اگر این مهم است، در سازنده‌ی هر لایه اعتبارسنجی کن و منبعِ واقعی را نام ببر.
