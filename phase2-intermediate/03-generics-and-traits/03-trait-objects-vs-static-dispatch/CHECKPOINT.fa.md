# ایست بازرسی

۱. چرا نمی‌توان `total_summary_length_generic::<T>` را با sliceای شامل هم `AnimeSeries` و هم `MangaVolume` صدا زد، با اینکه هر دو `Summarize` را پیاده می‌کنند؟ دقیقاً کدام ویژگی `T` این کار را ناممکن می‌کند؟

۲. با زبان خودت بگو vtable چیست. Rust آن را برای `Box<dyn Summarize>` هنگام compile آماده می‌کند یا runtime؟ فراخوانی `.summary()` روی آن مقدار در runtime چه کار اضافه‌ای انجام می‌دهد که فراخوانی در `total_summary_length_generic` لازم ندارد؟

۳. `make_mixed_collection` مقدار `Vec<Box<dyn Summarize>>` برمی‌گرداند، نه `Vec<AnimeSeries>`. اگر آن را `Vec<dyn Summarize>` بدون `Box` بنویسی چه می‌شود؟ به اندازه‌ی دقیقی فکر کن که `Vec` برای قراردادن عضوها کنار هم در backing array لازم دارد و به چیزی که درباره‌ی اندازه‌ی خود `dyn Summarize` می‌داند.

۴. یک مثال concrete بیرون از این درس بزن که آگاهانه `dyn Trait` را به جنریک ترجیح بدهی. چه ویژگی آن سناریو مشخصاً trait object را ضروری می‌کند؟

۵. گفتیم جنریک معمولاً از trait object سریع‌تر است. این تفاوت دقیقاً از کجا می‌آید؟ فراخوانی جنریک در runtime چه کاری **انجام نمی‌دهد** که فراخوانی `dyn Trait` باید انجام دهد؟
