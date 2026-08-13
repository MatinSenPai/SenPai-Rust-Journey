# کریت `taskforge-cli`

یه کلاینت CLI مبتنی بر `clap` برای `taskforge-api` — دومین کلاینتِ سبکِ مستقل (کنار `taskforge-admin-bot`) روی دقیقاً همون API، که الگوی "کلاینتِ سبک، هسته‌یِ سنگین" رو نشون می‌ده: هیچ‌کدوم از کلاینت‌ها هیچ منطقِ واقعیِ دامنه‌ای رو تو خودشون ندارن، فقط فراخوانی‌های HTTP و فرمت‌دهیِ خروجیِ ترمینال.

```sh
export TASKFORGE_API_URL=http://localhost:8080
export TASKFORGE_API_TOKEN=your-token

taskforge enqueue send_email --payload '{"to": "a@b.com"}'
taskforge list --job-type send_email --limit 10
taskforge get <job-id>
taskforge cancel <job-id>
```

## ران کردن تست‌ها

```sh
cargo test -p taskforge-cli
```

منطقِ فرمت‌دهیِ ترمینال تو فایل `src/format.rs` بدون هیچ I/Oای کاملاً یونیت‌تست شده؛ فراخوانی‌های HTTP تو `src/client.rs` نیازمندِ یه `taskforge-api`ِ در حال اجرا هستن و تو تست‌های پیش‌فرض اجرا نمی‌شن (دقیقاً مثل `taskforge-admin-bot`).

## یه هم‌پوشانی و تکرارِ کاملاً آگاهانه

فایل `src/client.rs` تو این کریت و فایل `taskforge-admin-bot/src/client.rs` دو تا کلاینتِ مجزا و کوچیکِ مبتنی بر `reqwest` هستن به جای اینکه یه کریتِ مشترک به اسم `taskforge-client` باشن. با داشتن ۴۰ تا ۶۰ خط کد تو هر کدوم، بیرون کشیدنِ یه کریتِ مشترک واسه فقط دو تا جا یه انتزاعِ زودرس (premature abstraction) محسوب می‌شد — این کار زمانی ارزشِ بررسی داره که کلاینت سومی پیدا بشه (مثلاً یه داشبورد وب یا یه ربات اسلاک)، نه قبل از اون.
