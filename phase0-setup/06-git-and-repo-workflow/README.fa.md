# ۰۶ — گردش کار Git و مخزن

این درس Rust جدیدی ندارد؛ هدف این است که چند ماه دیگر هم دقیقاً بدانی کجای مسیر هستی.

## سه مرجع اصلی

- `docs/conventions.md`: قواعد ساختار درس و workspace.
- `PROGRESS.md`: checklist اصلی و «در حال کار روی» فعلی.
- `README.md` هر فاز: فهرست درس‌های همان فاز.

## قرارداد commit

برای هر درس کامل یک commit با محدوده نام بسته بساز:

```text
feat(p1-02-01-move-semantics): complete exercise
```

checkbox همان درس را در `PROGRESS.md` در همان commit تغییر بده تا تاریخچه و checklist از هم جدا نشوند.

```sh
git add phase1-fundamentals/02-ownership-and-memory/01-move-semantics PROGRESS.md
git commit -m "feat(p1-02-01-move-semantics): complete exercise"
```

## پایان فاز

```sh
git tag -a phase1-complete -m "Finished Phase 1: Fundamentals"
git push origin phase1-complete
```

برخی درس‌های فقط‌خواندنی `Cargo.toml` ندارند و عمداً عضو workspace نیستند. برای تمرین کار تیمی می‌توانی برای هر فاز PR بسازی، اما برای مسیر انفرادی اجباری نیست.

```senpai-visual
{"kind":"roadmap","labels":["lesson","commit","phase tag"]}
```
