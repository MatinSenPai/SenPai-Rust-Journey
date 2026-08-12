# ۰۱ — مجموعه‌ها

Python معمولاً `list` و `dict` منعطف می‌دهد. Rust مجموعه‌ای از مجموعه‌ها با تضمین‌های متفاوت performance و ordering دارد و انتخاب شکل درست را صریح می‌کند. `HashSet` به‌جای deduplicate دستی `Vec` و `VecDeque` به‌جای `remove(0)` تکراری micro-optimization نیست؛ بیان درست مسئله است.

هر دو درس با داده‌ی تماشای انیمه/سریال کار می‌کنند:

1. [`Vec` و `HashMap`](01-vec-and-hashmap/README.md)
2. [`BTreeMap`، `HashSet` و `VecDeque`](02-btreemap-hashset-vecdeque/README.md)

در بک‌اند، `HashMap` lookup سریع، `BTreeMap` ترتیب کلید، `HashSet` یکتایی و `VecDeque` push/pop دو سر را مدل می‌کنند؛ انتخاب نهایی به workload و قرارداد نیاز دارد.

```senpai-visual
{"kind":"concept","labels":["Vec","HashMap","VecDeque"]}
```
