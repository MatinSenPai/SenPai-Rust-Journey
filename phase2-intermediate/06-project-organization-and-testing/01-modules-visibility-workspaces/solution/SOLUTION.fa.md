# پاسخ تشریحی

`pub(crate)` یعنی همه‌ی moduleهای همین crate مقدار را می‌بینند، اما crate وابسته نه. دسترسی بیرونی به `internal_rating` با خطای compile مربوط به field خصوصی رد می‌شود؛ هیچ بررسی runtimeای وجود ندارد.

privacy در Rust اجازه می‌دهد itemهای یک module برای خود module و فرزندانش دیده شوند و parent بتواند ساختار فرزندان را هماهنگ کند. module تست که فرزند ریشه‌ی crate است می‌تواند از مسیر مناسب به implementation داخلی دسترسی داشته باشد؛ integration test بیرون crate نمی‌تواند.

`pub use` یک مسیر عمومی پایدار و کوتاه به همان item می‌سازد. تعریف هنوز در `catalog` است، اما کاربر به layout داخلی وابسته نمی‌شود. نویسنده می‌تواند بعداً module را بازآرایی کند و فقط re-export را ثابت نگه دارد. همین اصل درباره‌ی `public_rating_band` صدق می‌کند: representation و thresholdهای داخلی می‌توانند عوض شوند، مادامی که قرارداد عمومی سه band حفظ شود.
