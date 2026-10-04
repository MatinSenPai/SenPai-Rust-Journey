# ۳.۷.۵ — مدل‌سازیِ RBAC و مجوزها

## در یک نگاه

بعد از این درس می‌توانی:

- «چه کسی چه کاری می‌تواند بکند» را با سه نوعِ ساده مدل کنی (کاربر نقش دارد، نقش مجوز می‌دهد) و با یک تابعِ خالص به نامِ `can` جواب بدهی، تابعی که بدونِ سرور هم تستش می‌کنی.
- توضیح بدهی چرا `401` و `403` و `404` از سه پرسشِ جدا می‌آیند («تو کی هستی؟»، «این وجود دارد؟»، «اجازه داری؟») و بررسی‌ها را به همین ترتیب بچینی.
- یک باگِ کنترلِ دسترسیِ شکسته (IDOR) را پیدا و درست کنی: هندلری که فقط بررسی می‌کند *وارد شده‌ای*، ولی هیچ‌وقت نمی‌پرسد نقدِ *چه کسی* را ویرایش می‌کنی.
- در `axum` نسخه‌ی ۰٫۸ یک اکسترکتورِ `RequirePermission<P>` بنویسی که بررسیِ نقش را به یک آرگومان در امضایِ هندلر تبدیل می‌کند.

**زمان:** حدود ۸۰ دقیقه · **پیش‌نیاز:**
[۳.۱.۳ — چیزهایی از HTTP که باید بدانی](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md)،
[۳.۲.۲ — نوشتنِ اکسترکتورِ خودت (`FromRequestParts`)](../../02-axum-and-rest-api-design/02-writing-your-own-extractor/README.fa.md)،
[۳.۷.۳ — JWT و میان‌افزار در `tower`](../03-jwt-and-tower-middleware/README.fa.md)

---

## چرا اهمیت دارد

بقیه‌ی این ماژول به پرسشِ *چه کسی تماس می‌گیرد* جواب داد: هشِ رمزی که می‌شود راستی‌آزمایی‌اش کرد، سشن یا توکنی که گواهی را حمل می‌کند، میان‌افزاری که در هر درخواست آن را راستی‌آزمایی می‌کند. این درس به پرسشِ بعدی جواب می‌دهد، همان که در سرویس‌هایِ واقعی از همه بیشتر غلط پیاده می‌شود: **این تماس‌گیرنده اجازه‌ی چه کاری دارد؟**

در جنگو جوابش را بدونِ نوشتن گرفته بودی. `Group` و `Permission` جدول‌اند، `user.has_perm("reviews.change_review")` از آن‌ها می‌پرسد، و `@permission_required` یا یک خطِ `permission_classes = [...]` در DRF ویو را نگهبانی می‌کند. در `axum` تا خودت نسازی چیزی نیست. این فرصتی است تا ببینی ایده چقدر کوچک است. نقش‌ها و مجوزها یک جدولِ جست‌وجو هستند، و «آیا این کاربر می‌تواند این نقد را ویرایش کند؟» یک تابعِ بولی است. بخشِ سخت کد نیست. یادت ماندن است که در *هر* هندلر بپرسی.

برای همین در فهرستِ ده‌گانه‌ی OWASP، **A01 یعنی Broken Access Control** در جایگاهِ اول است. شکلِ معمولش یک اکسپلویتِ پیچیده نیست. هندلری است که بررسی می‌کند وارد شده‌ای و بعد هر چه URL گفته انجام می‌دهد. `/reviews/1` را به `/reviews/2` عوض کنی، داری داده‌ی آدمِ دیگری را ویرایش می‌کنی. این درس مدل را می‌سازد، اکسترکتور را می‌سازد، و عادتی را که جلویش را می‌گیرد.

---

## مفهوم

### احرازِ هویت می‌پرسد «کی»، مجوزسنجی می‌پرسد «اجازه داری؟»

روی هر درخواستِ محافظت‌شده دو پرسشِ مختلف اجرا می‌شود، و با کدهایِ وضعیتِ مختلف شکست می‌خورند. این جفت را در ۳.۱.۳ دیده‌ای:

- **احرازِ هویت (authentication)**: این کیست؟ اعتبارنامه‌ی قابل‌استفاده نباشد، جواب `401 Unauthorized` است. اسمش گمراه‌کننده است، چون واقعاً یعنی *احرازِ هویت نشده*.
- **مجوزسنجی (authorization)**: این کاربر شناخته‌شده است. آیا اجازه دارد *این کار* را بکند؟ جوابِ «نه» می‌شود `403 Forbidden`.

بقیه‌ی ماژول پرسشِ اول را ساخت. این درس پرسشِ دوم است. در زبانِ جنگو، `request.user` جوابِ اولی است و `has_perm` دومی. `permission_classes`ِ DRF هم به همین دلیل بعد از کلاس‌هایِ احرازِ هویتش اجرا می‌شود.

پرسشِ سومی هم هست که میانِ این دو می‌نشیند و ترتیبِ بررسی‌ها را تعیین می‌کند. «نقدِ ۹۹ وجود دارد؟» جوابش `404` است. هندلرهایِ این درس سه پرسش را به این ترتیب می‌پرسند: کی، بعد چی، بعد اجازه داری. بخشِ «بررسی‌ها کجا می‌نشینند» دلیلش را می‌گوید.

### سه نوعِ ساده: مجوز، نقش، کاربر

از داده شروع کن، بدونِ HTTP. **مجوز (permission)** یک کار است که تماس‌گیرنده اجازه‌ی انجامش را دارد. **نقش (role)** یک بسته‌ی نام‌دار از مجوزهاست. کاربر مجموعه‌ای از نقش‌ها دارد. مجوزهایِ مؤثرش اجتماعِ چیزی است که هر نقش می‌دهد:

```rust
enum Role { Member, Moderator, Admin }
enum Permission { ReviewCreate, ReviewEditAny, ManageUsers }

// the role table: which permissions each role grants
let table: HashMap<Role, Vec<Permission>> = /* Member -> [ReviewCreate], ... */;
// a user's effective permissions: the union over their roles
roles.iter().flat_map(|role| table[role].iter().copied()).collect()
```

`examples/01-role-table.rs` جدول را می‌سازد و سه کاربر را چاپ می‌کند:

```text
alice    roles=[Member]  can={ReviewCreate}
carol    roles=[Member, Moderator]  can={ReviewCreate, ReviewEditAny}
mallory  roles=[]  can={}
```

به carol نگاه کن. هم ناظر است و هم عضو، پس هر دو نقش را دارد و هر دو مجموعه را می‌گیرد. نقش‌ها اینجا فقط اضافه می‌کنند و هیچ‌کدام چیزی کم نمی‌کند. ناظری که باید نقد هم بنویسد `Member` هم دارد، و سرورِ این درس دقیقاً همین کار را می‌کند. mallory هیچ نقشی ندارد. کاربرِ واقعی و شناخته‌شده است و هیچ اجازه‌ای ندارد. این وضعیتِ مجاز و پیش‌فرضِ ایمن است.

```senpai-visual
{"kind":"concept","labels":["کاربر نقش دارد: member و moderator","نقش مجوز می‌دهد: member -> create، moderator -> edit any","مجوزهایِ مؤثر = اجتماعِ نقش‌هایِ کاربر","کاربرِ بدونِ نقش هیچ کاری نمی‌تواند بکند","کد مجوز را بررسی می‌کند، نقش راهِ رسیدنِ آدم‌ها به مجوز است"]}
```

چیزی که کدت بررسی می‌کند مجوز است. نقش راهی است که آدم‌ها مجوز را *می‌گیرند*. هندلر باید بپرسد «آیا این کاربر می‌تواند هر نقدی را ویرایش کند؟»، نه «آیا این کاربر ناظر است؟»، چون اولی روزی هم که نقشِ `Support` اضافه کنی و او هم بتواند نقد را ویرایش کند درست می‌ماند.

نسخه‌ی جنگو از این‌ها `Permission` است (برایِ هر مدل خودکار ساخته می‌شود: `add_review`، `change_review`، `delete_review`، `view_review`)، `Group` (نقش) و `user.groups`. دو تفاوت مهم است. مجوزهایِ جنگو **برایِ هر مدل** هستند، نه برایِ هر شیء. و بک‌اندِ پیش‌فرضِ جنگو به هر بررسیِ سطحِ شیء `has_perm(perm, obj)` جوابِ `False` می‌دهد. پس جمله‌ی «کاربر می‌تواند نقدِ *خودش* را ویرایش کند» همیشه چیزی است که خودت می‌نویسی. بخشِ بعد همین را می‌نویسد.

### `can`: یک تابعِ خالص که تصمیم می‌گیرد

مالکیت در جدولِ نقش‌ها جا نمی‌شود. «ناظرها هر نقدی را می‌توانند ویرایش کنند» یک واقعیتِ نقش است. «عضوها می‌توانند نقدِ *خودشان* را ویرایش کنند» به یک نقدِ *مشخص* بستگی دارد. پس مدل هر مجوزِ ویرایش را دو تکه می‌کند: `ReviewEditOwn` و `ReviewEditAny`. بعد یک تابع جدول را با نویسنده‌ی نقد ترکیب می‌کند:

```rust
fn can_edit(roles: &[Role], user_id: u64, owner: Option<u64>) -> bool {
    roles.contains(&Role::Moderator)
        || (roles.contains(&Role::Member) && owner == Some(user_id))
}
```

`examples/03-can-matrix.rs` از هر یک از سه کاربر دو پرسش می‌پرسد:

```text
user                      own review      alice's review
alice (member, id 1)      true            true
bob (member, id 2)        true            false
carol (moderator, id 3)   true            true
```

`false`ِ ردیفِ bob کلِ ماجراست. او نقشِ عضو دارد، و آن نقش واقعاً اجازه‌ی ویرایشِ نقد می‌دهد، ولی فقط نقدِ خودِ عضو. تابعِ واقعیِ `can(policy, user, action, owner)` در درس همین شکل را دارد. سیاست، کاربر، یک `Action` (یعنی `Create` یا `Edit` یا `Delete`) و `owner: Option<UserId>` را می‌گیرد، که نویسنده‌ی همان نقد است یا `None`. `None` با هیچ‌کس برابر نیست، پس تماس‌گیرنده‌ای که مالک را نمی‌داند `false` می‌گیرد، نه یک `true`ِ تصادفی.

تابع **خالص (pure)** است: نه درخواست، نه پایگاه‌داده، نه ساعت. این تصمیمِ طراحی ارزشِ کپی‌کردن دارد. هر قاعده‌ی سیاستِ مجوزسنجیِ تو را می‌شود با مقدارهایِ ساده، در چند میکروثانیه، در یک جدول تست کرد. `axum` فقط راهی برایِ رساندنِ پرسش و جواب است.

### کنترلِ دسترسیِ شکسته: IDOR

حالا باگی را ببین که این تابع برایِ جلوگیری از آن هست. `examples/02-idor-missing-owner-check.rs` یک `edit_review` است که بررسی می‌کند تماس‌گیرنده وارد شده و هیچ چیزِ دیگر. کامپایل می‌شود و اجرا می‌شود:

```rust
fn edit_review(reviews: &mut HashMap<u64, Review>, logged_in: Option<u64>, id: u64, text: &str) -> u16 {
    if logged_in.is_none() {
        return 401;
    }
    match reviews.get_mut(&id) {
        Some(review) => { review.text = text.to_string(); 200 }
        None => 404,
    }
}
```

```text
bob (id 2) edits review 1: 200
review 1 now says "this anime is trash", author still 1
```

این یک **IDOR** است، یعنی *ارجاعِ مستقیمِ ناامنِ شیء* (insecure direct object reference): درخواست شیء را مستقیم اسم می‌برد (`/reviews/1`) و سرور هیچ‌وقت نمی‌پرسد این تماس‌گیرنده اجازه‌ی دست‌زدن به *آن* شیء را دارد یا نه. شکلِ استانداردِ OWASP A01 است. هیچ خطایی هیچ‌جا نمی‌آید. کد یک قابلیتِ کامل و کارای «کاربرانِ واردشده می‌توانند نقد را ویرایش کنند» است، و این قابلیتی نیست که کسی قصدش را داشته باشد. فقط تستی با کاربرِ اشتباه پیدایش می‌کند، و برای همین تست‌هایِ این درس همیشه یکی از آن‌ها را دارند.

درمانش یک فریم‌ورکِ جدید نیست. نویسنده‌ی نقد را به `can` می‌دهی. هندلری که به یک شیء دست می‌زند باید شیء را بخواند، مالکش را بگیرد، و بپرسد.

### اکسترکتور برایِ بررسیِ سطحِ نقش

بعضی مسیرها اصلاً شیء ندارند. `POST /users/{id}/roles` مخصوصِ ادمین‌هاست، همین. برایِ این‌ها تکرارِ `if !can(...) { return Err(Forbidden) }` در بالایِ هر هندلر همان مشکلِ کپی‌پیستی است که ۳.۲.۲ با اکسترکتور حل کرد، و همان ابزار جواب می‌دهد. `examples/04-require-permission-extractor.rs` نسخه‌ی مستقلش است:

```rust
struct Require<P>(PhantomData<P>);

impl<P: Needs, S: Send + Sync> FromRequestParts<S> for Require<P> {
    type Rejection = StatusCode;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, StatusCode> {
        let header = parts.headers.get("x-permissions").ok_or(StatusCode::UNAUTHORIZED)?;
        let held: HashSet<&str> = header.to_str().unwrap_or("").split(',').collect();
        if held.contains(P::PERMISSION) { Ok(Require(PhantomData)) } else { Err(StatusCode::FORBIDDEN) }
    }
}

async fn admin_page(_: Require<ManageUsers>) -> &'static str { "welcome, admin" }
```

```text
x-permissions: (absent)             -> 401 Unauthorized
x-permissions: create               -> 403 Forbidden
x-permissions: create,manage_users  -> 200 OK
```

سه چیز را ببین. `Require<ManageUsers>` در امضا *خودِ* قاعده‌ی مجوزسنجی است: خواننده آن را رویِ تابع می‌بیند و `axum` بررسی را پیش از بدنه‌ی هندلر اجرا می‌کند. `P` فقط در نوع می‌آید، و `PhantomData<P>` چیزی است که اجازه می‌دهد struct بدونِ فیلدی از آن نوع پارامترِ نوع را حمل کند. ردِ درخواست دقیقاً همان‌طور که ۳.۱.۳ یاد داد تقسیم می‌شود: بدونِ اعتبارنامه `401`، اعتبارنامه‌ی ناکافی `403`. (مثال مجوزها را مستقیم از یک هدر می‌خواند، که در دنیای واقعی حفره‌ی امنیتی است. کریتِ درس کاربر را از روی یک توکنِ bearer پیدا می‌کند، که جایگزینِ JWTِ راستی‌آزمایی‌شده‌ی ۳.۷.۳ است.)

`RequirePermission<P>`ِ کریت همین ایده روی مدلِ واقعی است: اول `Authenticated` را بیرون می‌کشد، بعد `Policy::allows(user, P::PERMISSION)` را می‌پرسد، و `User` را برمی‌گرداند تا هندلر بتواند از آن استفاده کند.

### بررسی‌ها کجا می‌نشینند

اکسترکتور پیش از هندلر اجرا می‌شود، پس فقط هدرهایِ درخواست را می‌شناسد و بس. نمی‌تواند بداند `/reviews/1` کدام نقد است. این تقسیمِ کار را تعیین می‌کند:

```senpai-visual
{"kind":"network","labels":["درخواست","Authenticated: توکنِ قابل‌استفاده نیست -> 401","RequirePermission: هیچ نقشی مجوز نمی‌دهد -> 403 (مسیرهایِ سطحِ نقش)","هندلر شیء را می‌خواند: نبود -> 404","can(user, action, owner): مجاز نیست -> 403","انجامِ کار -> 200 یا 204"]}
```

اکسترکتور **دروازه‌ی درشت** است («آیا نقش‌هایت اصلاً اجازه‌ی این *نوع* کار را می‌دهند؟»). `can` داخلِ هندلر **بررسیِ ریز** است («آیا اجازه داری این کار را روی *این* شیء بکنی؟»). ترتیبی که درس برایِ مسیرهایِ شیء‌دار انتخاب می‌کند `401`، بعد `404`، بعد `403` است. دلیلش: وقتی نقد نیست مالکی هم نیست که با آن مقایسه شود، پس «اجازه داری؟» هنوز جوابی ندارد. یک سؤالِ طراحیِ واقعی همین‌جاست. برایِ داده‌ی *عمومی* مثلِ نقدها، `403` که تأیید کند «نقدِ ۱ هست ولی مالِ تو نیست» چیزی لو نمی‌دهد. برایِ داده‌ی *خصوصی* خیلی از سرویس‌ها به هر کسی که اجازه‌ی دیدنِ شیء را ندارد `404` می‌دهند، تا `403` هرگز وجودش را تأیید نکند. یکی را عمداً انتخاب کن و بنویسش.

### نقش‌ها در ذخیره‌گاه، و نقش‌ها در توکن

همه‌ی این‌ها تا اینجا در حافظه بود. در پایگاه‌داده همین سه ایده سه جدول می‌شوند. این یک طرح است برایِ نشان‌دادنِ شکل، و خودِ کارِ اسکیما به [۳.۵.۲ — مایگریشن‌ها](../../05-postgres-and-sqlx/02-migrations/README.fa.md) و [۳.۶.۳ — طراحیِ اسکیما برایِ یک سرویسِ واقعی](../../06-database-design-and-query-performance/03-schema-design-for-a-real-service/README.fa.md) مربوط است:

```sql
-- illustrative sketch, not run in this lesson
CREATE TABLE roles (id SERIAL PRIMARY KEY, name TEXT NOT NULL UNIQUE);
CREATE TABLE role_permissions (
    role_id    INT  NOT NULL REFERENCES roles (id),
    permission TEXT NOT NULL,
    PRIMARY KEY (role_id, permission)
);
CREATE TABLE user_roles (
    user_id BIGINT NOT NULL REFERENCES users (id),
    role_id INT    NOT NULL REFERENCES roles (id),
    PRIMARY KEY (user_id, role_id)
);
```

ببین که نامِ مجوزها متن می‌مانند. `enum Permission` در Rust چیزی است که کامپایلر را وادار می‌کند هر `match`ای را که یادت رفته به‌روز کنی فهرست کند (اولین خطایِ پایین)، پس شکلِ معمول این است: نقش‌ها و بخشش‌هایشان در پایگاه‌داده، مجوزها به‌صورتِ enum در کد.

پرسشِ دوم این است که *نقش‌ها در لحظه‌ی درخواست از کجا خوانده می‌شوند*. اگر داخلِ یک توکنِ امضاشده بروند، ارتقا یا تنزلِ نقش تا وقتی آن توکن عوض نشود اثر نمی‌کند. در این درس `AppState::user_for_token` نقش‌هایِ کاربر را در هر درخواست از ذخیره‌گاه می‌خواند، پس `POST /users/{id}/roles` از همان تماسِ بعدی اثر دارد. پایین می‌بینی که اتفاق می‌افتد. این تازگی همان بده‌بستانی است که درس‌هایِ توکنِ پیشِ این در همین ماژول درباره‌اش بودند، و دلیلِ وجودِ توکن‌هایِ دسترسیِ کوتاه‌عمر با چرخشِ توکنِ رفرش (۳.۷.۴).

---

## دست‌به‌کد

سرورِ درس `src/main.rs` است، روی پورتِ `3210`، با پنج کاربرِ از پیش ساخته: alice (عضو)، bob (عضو)، carol (عضو و ناظر)، dave (عضو، ناظر و ادمین) و mallory (بدونِ نقش). توکن‌هایِ bearerشان `alice-token`، `bob-token` و همین‌طور بقیه است. تا پله‌هایِ «پیاده‌سازی» و «بساز» را انجام نداده‌ای، توابعِ `todo!()`ِ اسکلت پنیک می‌کنند، پس اول مثال‌ها را اجرا کن:

```sh
cargo run -p p3-07-05-modelling-rbac-and-permissions --example 01-role-table
cargo run -p p3-07-05-modelling-rbac-and-permissions --example 02-idor-missing-owner-check
cargo run -p p3-07-05-modelling-rbac-and-permissions --example 03-can-matrix
cargo run -p p3-07-05-modelling-rbac-and-permissions --example 04-require-permission-extractor
```

بعد آن سه‌تایی که کامپایل نمی‌شوند:

```sh
cargo build -p p3-07-05-modelling-rbac-and-permissions --example 05-permission-match-non-exhaustive-broken --features broken
cargo build -p p3-07-05-modelling-rbac-and-permissions --example 06-unused-type-parameter-broken --features broken
cargo build -p p3-07-05-modelling-rbac-and-permissions --example 07-role-not-hash-broken --features broken
```

وقتی «پیاده‌سازی» و «بساز» تمام شد، سرور را راه بینداز (`cargo run -p p3-07-05-modelling-rbac-and-permissions`) و با آن حرف بزن. خروجی‌هایِ پایین از کریتِ تمام‌شده‌ی `solution/` آمده‌اند، که همان روتر است. هر بلوکِ هدر با یک خطِ `date` تمام می‌شود که در هر اجرا فرق می‌کند. اول، بدونِ توکن:

```sh
curl -si http://127.0.0.1:3210/me
```

```text
HTTP/1.1 401 Unauthorized
content-type: application/json
www-authenticate: Bearer
content-length: 35
date: Sun, 04 Oct 2026 11:16:58 GMT

{"error":"authentication required"}
```

`401` با چالشِ `www-authenticate: Bearer`، تا کلاینت بداند چه نوع اعتبارنامه‌ای بفرستد. حالا bob، که عضو است، نقدِ alice را ویرایش می‌کند، و بعد carol، که ناظر است:

```sh
curl -si -X PATCH http://127.0.0.1:3210/reviews/1 -H 'authorization: Bearer bob-token' -H 'content-type: application/json' -d '{"text":"hacked"}'
```

```text
HTTP/1.1 403 Forbidden
content-type: application/json
content-length: 42
date: Sun, 04 Oct 2026 11:16:58 GMT

{"error":"you are not allowed to do that"}
```

```sh
curl -si -X PATCH http://127.0.0.1:3210/reviews/1 -H 'authorization: Bearer carol-token' -H 'content-type: application/json' -d '{"text":"[removed by moderator]"}'
```

```text
HTTP/1.1 200 OK
content-type: application/json
content-length: 69
date: Sun, 04 Oct 2026 11:16:58 GMT

{"id":1,"author":1,"anime":"Frieren","text":"[removed by moderator]"}
```

همان URL و همان بدنه یک بار `403` داد و یک بار `200`. فقط نقش‌هایِ تماس‌گیرنده فرق دارد. حالا مسیرِ سطحِ نقش. carol ناظر است ولی ادمین نیست، پس نمی‌تواند نقش بدهد. dave می‌تواند:

```sh
curl -si -X POST http://127.0.0.1:3210/users/2/roles -H 'authorization: Bearer carol-token' -H 'content-type: application/json' -d '{"role":"moderator"}'
```

```text
HTTP/1.1 403 Forbidden
content-type: application/json
content-length: 42
date: Sun, 04 Oct 2026 11:16:58 GMT

{"error":"you are not allowed to do that"}
```

```sh
curl -si -X POST http://127.0.0.1:3210/users/2/roles -H 'authorization: Bearer dave-token' -H 'content-type: application/json' -d '{"role":"moderator"}'
```

```text
HTTP/1.1 200 OK
content-type: application/json
content-length: 52
date: Sun, 04 Oct 2026 11:16:58 GMT

{"id":2,"name":"bob","roles":["member","moderator"]}
```

bob حالا ناظر است. همان درخواستی که لحظه‌ای پیش رد شد فوراً قبول می‌شود، چون نقش‌ها در هر درخواست جست‌وجو می‌شوند و در توکن یخ نزده‌اند:

```sh
curl -si -X PATCH http://127.0.0.1:3210/reviews/1 -H 'authorization: Bearer bob-token' -H 'content-type: application/json' -d '{"text":"bob is a moderator now"}'
```

```text
HTTP/1.1 200 OK
content-type: application/json
content-length: 69
date: Sun, 04 Oct 2026 11:16:58 GMT

{"id":1,"author":1,"anime":"Frieren","text":"bob is a moderator now"}
```

در آخر، ترتیبِ بررسی‌ها. alice اجازه دارد نقدِ خودش را ویرایش کند، ولی نقدِ ۹۹ وجود ندارد، پس جواب `404` است، نه `403`:

```sh
curl -si -X PATCH http://127.0.0.1:3210/reviews/99 -H 'authorization: Bearer alice-token' -H 'content-type: application/json' -d '{"text":"x"}'
```

```text
HTTP/1.1 404 Not Found
content-type: application/json
content-length: 21
date: Sun, 04 Oct 2026 11:16:58 GMT

{"error":"not found"}
```

بعد این‌ها را امتحان کن:

۱. در `01-role-table` یک نقشِ `Support` اضافه کن که `ReviewEditAny` می‌دهد و به یک کاربر `[Member, Support]` بده. مجموعه‌ی `can=`ش چه شکلی است؟ اگر هندلرها به‌جایِ مجوز `Role::Moderator` را بررسی می‌کردند چه چیزی را باید عوض می‌کردی؟
۲. در `03-can-matrix` کاربری بدونِ نقش اضافه کن. در ستونِ `own review` چه می‌شود، و آیا سیاستِ واقعی باید همین کار را بکند؟
۳. در `04-require-permission-extractor` هدرِ `x-permissions: manage_users_please` را بفرست. کدام وضعیت را می‌گیری، و چرا `200` نیست؟

---

## خطاهایی که خواهی دید

### `E0004`: یک `match` رویِ مجوزها که یکی را جا انداخته

```text
error[E0004]: non-exhaustive patterns: `&Permission::ManageUsers` not covered
  --> phase3-backend-foundations\07-auth-and-security\05-modelling-rbac-and-permissions\examples\05-permission-match-non-exhaustive-broken.rs:13:11
   |
13 |     match permission {
   |           ^^^^^^^^^^ pattern `&Permission::ManageUsers` not covered
   |
note: `Permission` defined here
  --> phase3-backend-foundations\07-auth-and-security\05-modelling-rbac-and-permissions\examples\05-permission-match-non-exhaustive-broken.rs:5:6
   |
 5 | enum Permission {
   |      ^^^^^^^^^^
...
 9 |     ManageUsers,
   |     ----------- not covered
   = note: the matched value is of type `&Permission`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
16 ~         Permission::ReviewEditOwn => false,
17 ~         &Permission::ManageUsers => todo!(),
   |

For more information about this error, try `rustc --explain E0004`.
error: could not compile `p3-07-05-modelling-rbac-and-permissions` (example "05-permission-match-non-exhaustive-broken") due to 1 previous error
```

**کامپایلر به چه ایراد می‌گیرد:** تابع سه‌تا از چهار مجوز را فهرست کرده و برایِ `ManageUsers` شاخه‌ای ندارد. `rustc` همه‌ی واریانت‌هایِ یک enum را می‌شناسد، پس `match`ای که وایلدکارد ندارد باید همه را اسم ببرد.

**درمان:** شاخه را اضافه کن و عمداً تصمیم بگیر جوابش چیست (`ManageUsers` ناظر لازم ندارد، پس `false`). `todo!()`ِ پیشنهادی را نچسبان و از شاخه‌ی `_ => false` هم دوری کن:

```rust
Permission::ManageUsers => false,
```

**چرا درمان همین است:** خودِ خطا قابلیت است. ماهِ بعد که مجوزی اضافه کنی، این خطا *همه‌ی* تابع‌هایی را فهرست می‌کند که حالا باید بگویند با آن چه می‌کنند. شاخه‌ی `_` آن فهرست را ساکت می‌کند، و روزی که کسی `ReviewDeleteAny` را اضافه کند وایلدکارد بی‌صدا به‌جایِ تو تصمیم می‌گیرد. در جدولِ مجوزسنجی این جای بدی برایِ پیش‌فرض است.

### `E0392`: پارامترِ نوعی که در هیچ فیلدی نیست

```text
error[E0392]: type parameter `P` is never used
 --> phase3-backend-foundations\07-auth-and-security\05-modelling-rbac-and-permissions\examples\06-unused-type-parameter-broken.rs:9:26
  |
9 | struct RequirePermission<P> {
  |                          ^ unused type parameter
  |
  = help: consider removing `P`, referring to it in a field, or using a marker such as `PhantomData`
  = help: if you intended `P` to be a const parameter, use `const P: /* Type */` instead

error[E0282]: type annotations needed
  --> phase3-backend-foundations\07-auth-and-security\05-modelling-rbac-and-permissions\examples\06-unused-type-parameter-broken.rs:16:5
   |
16 |     RequirePermission { user }
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ cannot infer type of the type parameter `P` declared on the struct `RequirePermission`
   |
help: consider specifying a concrete type for the type parameter `P`
   |
16 |     RequirePermission::</* Type */> { user }
   |                      ++++++++++++++

Some errors have detailed explanations: E0282, E0392.
For more information about an error, try `rustc --explain E0282`.
error: could not compile `p3-07-05-modelling-rbac-and-permissions` (example "06-unused-type-parameter-broken") due to 2 previous errors
```

**کامپایلر به چه ایراد می‌گیرد:** `P` فقط برایِ اسم‌بردنِ یک مجوز در نوع آمده. هیچ فیلدی از آن نوع نیست، پس `rustc` نمی‌تواند بفهمد struct برایِ `P`هایِ مختلف چه معنایی دارد و خودِ تعریف را گزارش می‌کند. خطایِ دوم دنباله‌ی اولی است: وقتی `P` بی‌استفاده است، چیزی در سازنده به `rustc` نمی‌گوید کدام `P` منظور است.

**درمان:** به struct فیلدی بده که `P` را نام ببرد بدونِ اینکه یکی نگه دارد:

```rust
struct RequirePermission<P> { user: User, _marker: PhantomData<P> }
```

**چرا درمان همین است:** `PhantomData<P>` هیچ حافظه‌ای نمی‌گیرد و می‌گوید «این struct جوری رفتار می‌کند که انگار به `P` بسته است». با `_marker: PhantomData` بساز. خطایِ اول را درست کنی، دومی هم می‌رود.

### `E0599`: استفاده از `Role` به‌عنوانِ کلیدِ `HashMap` بدونِ `Hash`

```text
error[E0599]: the method `insert` exists for struct `HashMap<Role, HashSet<&str>>`, but its trait bounds were not satisfied
  --> phase3-backend-foundations\07-auth-and-security\05-modelling-rbac-and-permissions\examples\07-role-not-hash-broken.rs:14:12
   |
 7 | enum Role {
   | --------- doesn't satisfy `Role: Eq` or `Role: Hash`
...
14 |     grants.insert(Role::Member, HashSet::from(["review:create"]));
   |            ^^^^^^
   |
   = note: the following trait bounds were not satisfied:
           `Role: Eq`
           `Role: Hash`
help: consider annotating `Role` with `#[derive(Eq, Hash, PartialEq)]`
   |
 7 + #[derive(Eq, Hash, PartialEq)]
 8 | enum Role {
   |

error[E0599]: the method `insert` exists for struct `HashMap<Role, HashSet<&str>>`, but its trait bounds were not satisfied
  --> phase3-backend-foundations\07-auth-and-security\05-modelling-rbac-and-permissions\examples\07-role-not-hash-broken.rs:15:12
   |
 7 | enum Role {
   | --------- doesn't satisfy `Role: Eq` or `Role: Hash`
...
15 |     grants.insert(Role::Moderator, HashSet::from(["review:edit_any"]));
   |            ^^^^^^
   |
   = note: the following trait bounds were not satisfied:
           `Role: Eq`
           `Role: Hash`
help: consider annotating `Role` with `#[derive(Eq, Hash, PartialEq)]`
   |
 7 + #[derive(Eq, Hash, PartialEq)]
 8 | enum Role {
   |

For more information about this error, try `rustc --explain E0599`.
error: could not compile `p3-07-05-modelling-rbac-and-permissions` (example "07-role-not-hash-broken") due to 2 previous errors
```

**کامپایلر به چه ایراد می‌گیرد:** نوشتنِ `HashMap<Role, ...>` مجاز است، ولی `insert` لازم دارد نوعِ کلید `Eq` و `Hash` داشته باشد، و این `Role` هیچ‌کدام را derive نکرده. خطا برایِ هر فراخوانی یک بار گزارش می‌شود، برای همین دو بار می‌بینی‌اش.

**درمان:** کاری را بکن که خطِ help می‌گوید:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
```

**چرا درمان همین است:** نقشه کلید را با هش‌کردن پیدا می‌کند و بعد مقایسه می‌کند، پس نوعِ کلید باید هش‌پذیر و برایِ برابری قابل‌مقایسه باشد. یک enumِ بدونِ فیلد مثلِ `Role` همه‌ی این‌ها را رایگان derive می‌کند.

### هیچ خطایی نمی‌آید: بررسیِ مالکِ جاافتاده

```text
bob (id 2) edits review 1: 200
review 1 now says "this anime is trash", author still 1
```

**چه چیزی واقعاً خراب است:** `examples/02-idor-missing-owner-check.rs` کامپایل می‌شود، اجرا می‌شود، و هر بار `200` برمی‌گرداند. بررسی می‌کند کسی وارد شده و هرگز تماس‌گیرنده را با نویسنده‌ی نقد مقایسه نمی‌کند. هیچ نوعی نمی‌تواند «این تماس‌گیرنده مالکِ این ردیف است» را بیان کند، پس هیچ کامپایلری نمی‌گیردش.

**درمان:** پیش از تغییر ببین نقد را چه کسی نوشته. اگر تماس‌گیرنده نویسنده نیست `403` بده:

```rust
if review.author != caller { return 403; }
```

**چرا درمان همین است:** تصمیم *دو* ورودی لازم دارد، تماس‌گیرنده و شیء. احرازِ هویت فقط اولی را دارد. در کریتِ درس این مقایسه در `can` است، و تنها راهِ یافتنِ باگ تستی است که کاربرِ اشتباه امتحانش می‌کند. هر قاعده‌ی مالکیتی که می‌نویسی باید همراهِ همان تست باشد.

---

## تمرین

### گرم‌کردن

<details>
<summary>کاربری که فقط نقشِ <code>Member</code> دارد، با توکنِ معتبر <code>POST /users/2/roles</code> را صدا می‌زند. <code>401</code> یا <code>403</code>؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`403 Forbidden`. توکن درست است، پس سرور می‌داند تماس‌گیرنده کیست. نقش‌هایش `ManageUsers` را شامل نمی‌شود. `401` برایِ «نتوانستم بشناسمت» است.

</details>

<details>
<summary>carol فقط <code>Moderator</code> دارد، نه <code>Member</code>. با سیاستِ استانداردِ درس، آیا می‌تواند نقد بسازد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

نه. `ReviewCreate` را `Member` می‌دهد و نقش‌ها فقط اضافه می‌کنند. برای همین carolِ از پیش ساخته هم `Member` دارد هم `Moderator`. هیچ‌کدام از دو نقش مجوزهایِ دیگری را شامل نمی‌شود.

</details>

<details>
<summary><code>can(&amp;policy, &amp;member, Action::Edit, None)</code> برایِ یک عضوِ ساده چه برمی‌گرداند، و برایِ یک ناظر چه؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

برایِ عضو `false` و برایِ ناظر `true`. `owner: None` با شناسه‌ی هیچ‌کس برابر نیست، پس مجوزِ «خودش» کمکی نمی‌کند، ولی ناظر `ReviewEditAny` دارد که اصلاً به مالک نگاه نمی‌کند.

</details>

<details>
<summary>bob (یک عضوِ ساده) <code>PATCH /reviews/99</code> می‌فرستد و نقدِ ۹۹ وجود ندارد. چه وضعیتی برمی‌گردد، و چرا <code>403</code> نه؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`404 Not Found`. این درس اول کی (`401`)، بعد چی (`404`)، بعد اجازه داری (`403`) را بررسی می‌کند. وقتی نقد نیست مالکی نیست که مقایسه شود، پس «اجازه داری؟» هیچ‌وقت پرسیده نمی‌شود.

</details>

### تعمیر

هر چهار را درست کن:

۱. `examples/05-permission-match-non-exhaustive-broken.rs` کامپایل شود، با یک شاخه‌ی واقعی برایِ `ManageUsers` و بدونِ وایلدکارد.
۲. `examples/06-unused-type-parameter-broken.rs` کامپایل شود و `4` چاپ کند.
۳. `examples/07-role-not-hash-broken.rs` کامپایل شود و `2` چاپ کند.
۴. `examples/02-idor-missing-owner-check.rs` برایِ bob عددِ `403` چاپ کند و متنِ نقد را عوض نکند. (این یکی از قبل کامپایل می‌شود. باگش در منطق است.)

### پیاده‌سازی

دو تابع در `src/lib.rs`، بدونِ هیچ HTTP:

```sh
cargo test -p p3-07-05-modelling-rbac-and-permissions --test policy_test
```

کامنتِ مستندِ هر کدام مشخصاتِ کاملش است:

- `Policy::allows`: آیا یکی از نقش‌هایِ کاربر این مجوز را می‌دهد؟
- `can`: آیا این کاربر می‌تواند این کنش را رویِ نقدی که `owner` نوشته انجام دهد؟ ساختن، ویرایش و حذف، با مجوزهایِ «خودش» و «هر».

### بساز

سرور را سرتاسر کار بینداز. چهار `todo!()`ِ دیگر در `src/lib.rs`: پیاده‌سازیِ `FromRequestParts` برایِ `RequirePermission<P>`، و هندلرهایِ `update_review`، `delete_review` و `grant_role`. ترتیبِ بررسی و کدهایِ وضعیت در کامنتِ مستندشان هست. ۱۹ تستِ `tests/api_test.rs` کلِ روتر را با `oneshot` می‌رانند:

```sh
cargo test -p p3-07-05-modelling-rbac-and-permissions --test api_test
```

### چالش (اختیاری)

سیستم‌هایِ واقعی اغلب اجازه می‌دهند نقشی از نقشِ دیگر *ارث* ببرد: `Admin` هر چه `Moderator` دارد را می‌گیرد، و او هر چه `Member` دارد. `challenge::effective_permissions` را پیاده کن. این تابع آنچه هر نقش مستقیم می‌دهد و یک نقشه از والدها را می‌گیرد، و هر مجوزی را که یک نقش دارد، با والدها، برمی‌گرداند. برایِ تنظیماتِ اشتباهی که دو نقش از هم ارث می‌برند آماده باش. تابع باید باز هم برگردد. این به پیمایشِ گراف سرک می‌کشد، ولی فقط یک مجموعه‌ی دیده‌شده‌ها و یک حلقه لازم است، و سه تستِ `src/lib.rs` آن را بررسی می‌کنند.

---

## جمع‌بندی

این درس ماژولِ ۳.۷ را می‌بندد. ماژول اعتمادِ یک درخواست را از اول تا آخر دنبال کرد. [۳.۷.۱ — هش‌کردنِ پسورد با argon2](../01-password-hashing-argon2/README.fa.md) چیزی را ذخیره کرد که ثابت می‌کند تو کی هستی. [۳.۷.۲ — Session در برابرِ JWT: مصالحه‌یِ واقعی](../02-sessions-vs-jwt/README.fa.md) دو راهی را که این گواهی با هر درخواست می‌رود سبک‌سنگین کرد. [۳.۷.۳ — JWT و میان‌افزار در `tower`](../03-jwt-and-tower-middleware/README.fa.md) آن را در هر تماس راستی‌آزمایی کرد. [۳.۷.۴ — چرخشِ refresh token و باطل‌سازیش](../04-refresh-token-rotation-and-revocation/README.fa.md) سراغِ این رفت که گواهی باید زودتر از موعد از کار بیفتد. این درس جواب می‌دهد هویتِ ثابت‌شده اجازه‌ی چه کاری دارد. ماژولِ ۳.۸ بعد از این می‌آید و به این می‌پردازد که APIِ تو وقتی رد می‌کند چه می‌گوید، و چطور همه‌ی این‌ها را در مقیاس تست کنی.

| اصطلاح | یعنی چه | کجا به کار می‌آید |
|---|---|---|
| احرازِ هویت (authentication) | مشخص‌کردنِ اینکه تماس‌گیرنده کیست؛ شکستش `401` است | هر مسیرِ محافظت‌شده |
| مجوزسنجی (authorization) | تصمیم‌گرفتن درباره‌ی اینکه تماس‌گیرنده‌ی شناخته‌شده چه کاری می‌تواند بکند؛ شکستش `403` است | هر مسیرِ محافظت‌شده |
| مجوز (permission) | یک کار که تماس‌گیرنده اجازه‌ی انجامش را دارد، مثلِ `ReviewEditAny` | چیزی که هندلرها و `can` بررسی می‌کنند |
| نقش (role) | بسته‌ی نام‌دارِ مجوزها که کاربران دارند | راهِ دادن و ممیزیِ مجوزها |
| RBAC (کنترلِ دسترسیِ مبتنی بر نقش) | مجوزها از نقش می‌آیند، نه از تک‌تکِ کاربران | بیشترِ سیستم‌هایِ واقعیِ کنترلِ دسترسی |
| `can` | یک تابعِ خالصِ بله/خیر از سیاست، کاربر، کنش و مالک | تست‌ها، هندلرها، هر جا |
| بررسیِ مالکیت (ownership check) | مقایسه‌ی تماس‌گیرنده با مالکِ شیء | ویرایش و حذف با شناسه |
| IDOR / کنترلِ دسترسیِ شکسته | کارکردن رویِ شیء با شناسه بدونِ بررسی؛ OWASP A01 | بازبینیِ هر هندلری که شناسه می‌گیرد |
| `RequirePermission<P>` | اکسترکتوری که تماس‌گیرنده‌ی بدونِ `P` را رد می‌کند | مسیرهایِ سطحِ نقش |

### الان می‌دانی

- احرازِ هویت و مجوزسنجی دو پرسش با دو کدِ وضعیت‌اند. کاربرِ شناخته‌شده‌ای که اجازه ندارد `403` می‌گیرد، نه `401`.
- کاربران نقش دارند، نقش‌ها مجوز می‌دهند، و مجوزهایِ مؤثر اجتماع است. هندلرها مجوز را بررسی می‌کنند، نه اسمِ نقش را.
- مالکیت در جدولِ نقش‌ها جا نمی‌شود، پس مجوزها دو شکلِ «خودش» و «هر» دارند، و یک تابعِ خالص، `can`، آن‌ها را با مالکِ شیء ترکیب می‌کند.
- هندلری که شناسه می‌گیرد و فقط بررسی می‌کند وارد شده‌ای IDOR دارد، و هیچ کامپایلری هشدار نمی‌دهد.
- اکسترکتورِ `FromRequestParts` جایِ درستِ بررسیِ درشتِ سطحِ نقش است. بررسیِ سطحِ شیء نمی‌تواند آنجا باشد، چون اکسترکتور شیء را نمی‌شناسد.
- جایی که نقش‌ها از آن خوانده می‌شوند سرعتِ اثرِ تغییر را تعیین می‌کند: از ذخیره‌گاه در هر درخواست، فوری. از توکنِ امضاشده، وقتی توکن عوض شود.

### بعداً کامل‌تر می‌بینی

- **یک شکلِ بدنه‌ی خطا برایِ `401` و `403` و `404` و بقیه، در هر هندلر** — [۳.۸.۱ — پاکت‌هایِ خطایِ یکدست](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)
- **تستِ یک سیاستِ مجوزسنجی روی پایگاه‌داده‌ی واقعی** — [۳.۸.۳ — تستِ یکپارچگی با `testcontainers`](../../08-error-handling-and-testing-at-scale/03-integration-tests-with-testcontainers/README.fa.md)
- **ساختنِ سریعِ کاربر و نقش و نقد برایِ تست‌ها** — [۳.۸.۴ — factory و fixture برایِ داده‌یِ تست](../../08-error-handling-and-testing-at-scale/04-test-data-factories-and-fixtures/README.fa.md)
- **ذخیره‌ی نقش‌ها و بخشش‌ها در جدول، با مهاجرت** — [۳.۵.۲ — مایگریشن‌ها](../../05-postgres-and-sqlx/02-migrations/README.fa.md)
- **طراحیِ جدول‌هایِ واقعی، جایی که طرحی مثلِ بالا قیدها و ایندکس‌هایش را می‌گیرد** — [۳.۶.۳ — طراحیِ اسکیما برایِ یک سرویسِ واقعی](../../06-database-design-and-query-performance/03-schema-design-for-a-real-service/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا کاربرِ شناخته‌شده‌ای که نقشِ درست را ندارد `403` می‌گیرد، در حالی که درخواستِ بدونِ توکن `401` می‌گیرد؟
- چرا «آیا این کاربر می‌تواند این نقد را ویرایش کند؟» تابعی از خودِ نقد است، نه فقط نقش‌هایِ کاربر؟
- IDOR چیست، و چرا کامپایلر نمی‌تواند پیدایش کند؟
- چرا `RequirePermission<P>` به `PhantomData` نیاز دارد، و چرا اکسترکتور نمی‌تواند بررسیِ مالکیت را انجام دهد؟
- چرا شاخه‌ی `_ => false` در یک `match` رویِ مجوزها اشتباه است، با اینکه کامپایل می‌شود؟
- اگر نقش‌ها در هر درخواست از جدولِ ذخیره‌شده خوانده شوند، به‌جایِ claimهایِ توکن، زمان‌بندیِ یک ارتقا چه فرقی می‌کند؟

---

## بیشتر

- [OWASP Top 10: A01 Broken Access Control](https://owasp.org/Top10/): دسته‌ای که IDOR این درس به آن تعلق دارد، با الگوهایِ شکستِ واقعی.
- [OWASP Authorization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html): پیش‌فرض رد، اجرا در هر درخواست، و تست‌کردنِ منطقِ مجوزسنجی.
- [Django: permissions and authorization](https://docs.djangoproject.com/en/stable/topics/auth/default/#permissions-and-authorization): `Permission` و `Group` و `has_perm`، مدلی که این درس با دست بازسازی‌اش کرد.
- [DRF: permissions](https://www.django-rest-framework.org/api-guide/permissions/): `has_permission` در برابرِ `has_object_permission`، همان تقسیمِ درشت و ریزِ اکسترکتور و `can`.
- [axum: `FromRequestParts`](https://docs.rs/axum/0.8/axum/extract/trait.FromRequestParts.html): صفتی که `RequirePermission` رویش سوار است.
