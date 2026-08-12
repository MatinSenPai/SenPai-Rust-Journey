# ۰۵.۲ — enum و `match`

enum دقیقاً یکی از چند شکل را مدل می‌کند:

```rust
enum WatchStatus {
    PlanToWatch,
    Watching { episode: u32 },
    Completed { rating: u8 },
    Dropped,
}
```

مدلی با وضعیت متنی و چند فیلد اختیاری می‌تواند حالت نامعتبر بسازد؛ مثلاً امتیاز برای اثری که هنوز در حال تماشا است. در enum، داده‌ی `episode` فقط در `Watching` و `rating` فقط در `Completed` وجود دارد. به این ساختار «نوع مجموع» (Sum Type) یا Tagged Union هم می‌گویند.

```rust
fn describe(status: &WatchStatus) -> String {
    match status {
        WatchStatus::PlanToWatch => "not started yet".to_string(),
        WatchStatus::Watching { episode } => format!("on episode {episode}"),
        WatchStatus::Completed { rating } => format!("finished, rated {rating}/10"),
        WatchStatus::Dropped => "dropped".to_string(),
    }
}
```

هر arm هم variant را تشخیص می‌دهد و هم داده را destructure می‌کند. افزودن variant جدید، تمام matchهای ناقص را به خطای کامپایل تبدیل می‌کند.

در بک‌اند، `JobStatus::Queued { position } | Running { started_at } | Failed { reason }` نسبت به string و ستون‌های nullable حالت‌های غیرممکن را حذف می‌کند. البته database migration و serialization compatibility هنوز باید جدا طراحی شوند.

```senpai-visual
{"kind":"result","labels":["Queued","Running","Completed"]}
```
