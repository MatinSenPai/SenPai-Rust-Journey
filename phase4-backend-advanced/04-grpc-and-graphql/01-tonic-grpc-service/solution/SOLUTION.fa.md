# راه‌حل

```rust
async fn create_note(
    &self,
    request: Request<CreateNoteRequest>,
) -> Result<Response<Note>, Status> {
    let req = request.into_inner();
    let mut state = self.state.lock().unwrap();

    let id = state.next_id.to_string();
    state.next_id += 1;

    let note = Note {
        id: id.clone(),
        title: req.title,
        body: req.body,
    };
    state.notes.insert(id, note.clone());

    Ok(Response::new(note))
}

async fn get_note(&self, request: Request<GetNoteRequest>) -> Result<Response<Note>, Status> {
    let req = request.into_inner();
    let state = self.state.lock().unwrap();

    match state.notes.get(&req.id) {
        Some(note) => Ok(Response::new(note.clone())),
        None => Err(Status::not_found(format!("note {} not found", req.id))),
    }
}

async fn list_notes(
    &self,
    _request: Request<ListNotesRequest>,
) -> Result<Response<ListNotesResponse>, Status> {
    let state = self.state.lock().unwrap();
    let notes: Vec<Note> = state.notes.values().cloned().collect();
    Ok(Response::new(ListNotesResponse { notes }))
}
```

## قالب و شکلِ یه سرویسِ ساخته‌شده با تولیدِ کُد (generated service)

عبارت `!("tonic::include_proto!("notes` میاد و کدهایی رو که فایل `build.rs` تو زمانِ کامپایل از روی فایلِ `proto/notes.proto` تولید کرده بود رو تو پروژه تزریق (pull in) می‌کنه (که این کار با استفاده از متدِ `tonic_build::compile_protos` انجام می‌شه، و اونم برای این کار از همون باینریِ `protoc` که کریتِ `protoc-bin-vendored` با خودش بنده‌بندی (bundled) کرده بود استفاده می‌کنه چون رو این محیط سندباکس کلاً یه دونه `protoc` نصب‌شده رو سیستم‌عامل نداریم — که این دقیقاً همون ترفندیه که می‌تونی واسه هر رانرِ CI (CI runner) یا کامپیوترِ هر هم‌تیمی دیگه‌ای که هنوز `protoc` رو جداگونه نصب نکرده به کار ببری). اون کدهای تولیدشده در واقع دو تا چیز بهت می‌دن: اول یه مشت ساختار ساده و متنی (plain data structs) مثل `Note`، `CreateNoteRequest` و غیره، که رسماً همون پیاده‌سازیِ خصیصه‌ی `Message` تو کتابخونه‌ی prost هستن — که تو gRPC دقیقاً معادلِ همون ساختارهاییه که صفتِ `[(derive(Serialize, Deserialize#]` رو داشتن — و دومی یه خصیصه‌ی `NotesService` (تو ماژول `notes_service_server`) که به ازایِ هر RPCای که تو فایل `.proto` بوده یه متدِ `async fn` توش وجود داره. پیاده‌سازی کردنِ اون خصیصه رویِ `NotesServiceImpl` کلاً کلِ وظیفه‌ی تو واسه نوشتن برنامه است؛ وجودِ ماکرویِ `[tonic::async_trait]#` هم فقط و فقط به این خاطره که زمانی که این کریت واسه اولین بار نوشته می‌شد، متدهایِ `async fn`ِ توکار (native) تو خصیصه‌ها هنوز از امنیتِ شیءِ داینامیک (`dyn`-safety) و مرزهای `Send` که tonic بهشون نیاز داشت پشتیبانی نمی‌کردن (البته الان دیگه اکوسیستم Rust کلاً داره از این ماکرو دست می‌کشه و دور می‌شه، اما کدهایی که واسه سرورها تولید می‌شن به خاطرِ حفظ سازگاریِ رو به عقب (backwards compatibility) کماکان دارن ازش استفاده می‌کنن).

## ساختار `<<Mutex<NotesState`، دقیقاً همون الگویِ تکراری تو همه‌جا

ساختارِ `NotesServiceImpl` تو دلِ خودش فقط یه دونه `<<Mutex<NotesState` رو نگه داشته که داره از یه `HashMap` درون‌حافظه‌ای (in-memory) و یه شمارنده‌ی آیدی مراقبت می‌کنه — دقیقاً همون الگو و روشِ درس‌هایِ ریسمان‌ها و قفل‌های تو فاز ۲ (درس `Mutex`) که البته اینجا دیگه پوسته‌یِ `Arc` کلاً حذف شده (دلیل اینکه اینجا کلاً `Arc` نیازی نیست اینه که وقتی تو متد `.add_service(...)` رو صدا می‌زنی، خودِ فریم‌ورکِ tonic میاد کُل سرویس رو می‌ذاره تو دلِ یه `Arc` که کاملاً تو دستِ خودشه، پس از نگاه فریم‌ورک፣ ارجاعِ `&self` از قبل کلاً مشترک و قابل کپی (shared-and-cloneable) هستش). تک‌تکِ متدها میان قفل رو می‌گیرن (take the lock)، کارشون رو انجام می‌دن، و تو پایانِ بلوک اجازه می‌دن قفل خودش خودبه‌خود آزاد بشه (drop) — دقیقاً همون انضباط و دیسیپلینی که تو ساختار `InMemoryQueue` تو درسِ صفِ اسباب‌بازیِ تو ماژول قبلی دیدی.

## خطاها مقادیری با طعم و عطرِ gRPC هستن (Errors as values)

بیایم متدِ `get_note` که خطایِ `Err(Status::not_found(...))` رو برمی‌گردونه رو مقایسه کنیم با متدِ `ApiError` از فریم‌ورکِ axum تو فاز ۳ که نوعِ `IntoResponse` رو پیاده‌سازی می‌کرد تا بتونه یه خطای دامنه (domain error) رو بگیره و اونو تبدیل کنه به ترکیبِ کُد وضعیت HTTP و یه بدنه‌یِ JSON. نوعِ `tonic::Status` دقیقاً معادلِ همون مکانیزم تو دنیای gRPC هستش: یه نوعِ خطا تو پایین‌ترین سطح شبکه (wire-level error type) که هم یه کُدِ `Code` رو با خودش می‌بره (یعنی یه enum با یه سری گزینه‌ها مثل `NotFound`، `InvalidArgument`، `Internal` — که هم‌سنگ و معادلِ همون وضعیت‌هایِ کُدِ HTTP تو gRPC هستن) و هم یه پیام متنیِ قابل‌خوندن برای انسان (human-readable message) رو به همراه داره. هر دویِ اینا در واقع همون ایده‌ی کاملاً یکسانی هستن که تا حالا سه بار با چشم دیدیشون (حالت `<Result<T, DomainError>` تو فازهای ۱ و ۲، حالت `<Result<T, ApiError>` تو axum، و در نهایت `<Result<Response<T>, Status>` تو همینجا): این ایده که سیستمِ نوع‌ها (type system) رو مجبور کنی تا فراخواننده‌ی (caller) سیستم رو تحت فشار بذاره که کُلِ وضعیت‌هایِ شکست رو صریحاً هندل کنه، و پروسه‌یِ ترجمه‌ی "چه اتفاقی تو سیستم خراب شد" به "کلاینت در نهایت قراره چی ببینه" رو تو یه نقطه‌یِ مرزی (boundary) کلاً متمرکز و یک‌جا (centralize) کنی، به جای اینکه بیای و خطوط کُد رو پُر کنی از `!panic` یا گذاشتن مقادیرِ پیش‌فرض (silent defaults) اونم دقیقاً وسط کدهای منطق کسب‌وکار (business logic).

## چرا بینِ فراخوانی‌هایِ همروندِ (concurrent) `create_note` کلاً هیچ تداخلی تو گرفتنِ `id`ها پیش نمیاد

متغیرِ `next_id` تنها و تنها زمانی خونده می‌شه و بالا می‌ره (incremented) که قفلِ `Mutex` کاملاً گرفته شده باشه، پس دو تا فراخوانیِ RPCِ همروند به `create_note` (با توجه به اینکه tonic هر ریکوئست رو رو یه تسکِ مجزایِ tokio ران می‌کنه) محاله بتونن یه مقدارِ کاملاً یکسان رو همزمان مشاهده کنن — درخواست دوم برای گرفتنِ قفل کاملاً بلوک (blocks) می‌شه تا زمانی که درخواست اول شمارنده رو یکی برده باشه بالا و قفل رو رها کرده باشه. این دقیقاً و مو به مو همون استدلالیه که تو ماژول تست‌ها برای تستِ `each_created_note_gets_a_distinct_id` آوردیم، و دقیقاً همون منطقیه که باعث شد وقتی تو درس قبلی از `Mutex` مختصِ یک پروسه رفتیم رو دیتابیسِ مشترکِ `PostgresQueue`، استفاده از دستورِ `FOR UPDATE SKIP LOCKED` تا این حد ضروری و واجب بشه — یه سرویسِ درون-حافظه‌ایِ تک-پروسه‌ای (single-process) می‌تونه با خیالِ راحت به `std::sync::Mutex` تکیه بده، دقیقاً و مشخصاً به این دلیل ساده که اونجا کلاً کُلِ چیزی که قراره ازش محافظت بشه، حافظه‌ی همون یک دونه پروسه‌ست و لاغیر.