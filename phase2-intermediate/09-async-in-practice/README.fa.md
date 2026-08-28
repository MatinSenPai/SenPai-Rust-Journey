# ۰۹ — async در عمل

`tokio-basics` نشانت داد که `async fn` کار می‌کند. این ماژول جایی است که با آن
می‌سازی: راه‌اندازی و ساختاردهیِ کارِ هم‌روندِ واقعی، لغوِ امنِ آن، و فهمیدنِ
اینکه یک تسک کِی به‌جایِ `.await` به `spawn_blocking` نیاز دارد.

۱. [`spawn`، `JoinSet`، هم‌روندیِ ساخت‌یافته](01-spawn-joinset-structured-concurrency/README.fa.md)
۲. [`select!` و ایمنیِ لغو](02-select-and-cancellation-safety/README.fa.md)
۳. [استریم‌ها: `futures::Stream` و `tokio-stream`](03-streams/README.fa.md)
۴. [صفت‌های async و `spawn_blocking`](04-async-traits-and-blocking/README.fa.md)
