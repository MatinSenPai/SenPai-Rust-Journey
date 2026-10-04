# راه‌حل — ۳.۷.۵ مدل‌سازیِ RBAC و مجوزها

کدِ کامل `solution/src/lib.rs` است. همه‌ی تست‌هایِ `solution/tests/` (۱۳ تستِ سیاست، ۱۹ تستِ روتر) و سه تستِ چالش داخلِ `lib.rs` را پاس می‌کند.

## تعمیر

۱. `05`: شاخه‌ی `Permission::ManageUsers => false,` را اضافه کن. وایلدکارد نگذار، تا مجوزِ بعدی دوباره این `match` را بشکند و کامپایلر فهرستش کند.
۲. `06`: فیلدِ `_marker: PhantomData<P>` را اضافه کن (با `use std::marker::PhantomData;`) و مقدار را با `_marker: PhantomData` بساز.
۳. `07`: `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]` رویِ `Role`.
۴. `02`: پیش از نوشتن به نقد نگاه کن. `review.author` را با شناسه‌ی تماس‌گیرنده مقایسه کن و اگر فرق دارند `403` بده. متنِ نقد باید همان بماند.

## `Policy::allows`

```rust
user.roles.iter().any(|role| {
    self.grants
        .get(role)
        .is_some_and(|set| set.contains(&permission))
})
```

برایِ هر نقشِ کاربر مجموعه‌ی همان نقش را پیدا می‌کنی. نقشی که سیاست برایش ورودی ندارد `None` می‌دهد و `is_some_and` آن را `false` می‌کند. کاربرِ بدونِ نقش یعنی `any` رویِ هیچ می‌چرخد، که `false` است. «اجتماع رویِ نقش‌ها» همان `any` است.

## `can`

```rust
let is_owner = owner == Some(user.id);
match action {
    Action::Create => policy.allows(user, Permission::ReviewCreate),
    Action::Edit => {
        policy.allows(user, Permission::ReviewEditAny)
            || (is_owner && policy.allows(user, Permission::ReviewEditOwn))
    }
    Action::Delete => { /* the same with the Delete permissions */ }
}
```

`owner == Some(user.id)` برایِ `None` برابر `false` است، پس «مالکِ ناشناخته با هیچ‌کس برابر نیست» بدونِ کدِ اضافه به دست می‌آید. مجوزِ «هر» اول بررسی می‌شود و به مالک نگاه نمی‌کند. مجوزِ «خودش» هم خودِ مجوز را می‌خواهد و هم تطابق را. `match` وایلدکارد ندارد، پس `Action`ِ تازه اینجا خطایِ کامپایل می‌شود.

## `RequirePermission`

```rust
let Authenticated(user) = Authenticated::from_request_parts(parts, state).await?;
if state.policy().allows(&user, P::PERMISSION) {
    Ok(RequirePermission { user, _marker: PhantomData })
} else {
    Err(ApiError::Forbidden)
}
```

استفاده‌ی دوباره از `Authenticated` یعنی همه‌ی حالت‌هایِ `401` در یک جا تصمیم گرفته می‌شود و `?` ردِ درخواستش را بدونِ تغییر عبور می‌دهد. فقط کاربرِ شناخته‌شده به `403` می‌رسد.

## هندلرها

`update_review` و `delete_review` ذخیره‌گاه را قفل می‌کنند، نقد را پیدا می‌کنند (اول `NotFound`)، بعد `can(.., Some(review.author))` را صدا می‌زنند و برایِ `false` جوابِ `Forbidden` می‌دهند. آرگومانِ `Authenticated` اول اجرا می‌شود، پس ترتیب `401`، `404`، `403` است. `grant_role` اولین آرگومانش `RequirePermission<CanManageUsers>` است، پس `401` و `403` پیش از دست‌زدن به مسیر و بدنه اتفاق می‌افتند، و بعد کاربر را پیدا می‌کند (`404`) و نقش را در مجموعه می‌گذارد.

توجه کن که `update_review` قفل را هم در بررسی و هم در نوشتن نگه می‌دارد. وقتی بررسی و نوشتن زیرِ یک قفل‌اند، تغییرِ نقش نمی‌تواند میانشان بپرد.

## چالش

```rust
let mut seen = HashSet::new();
let mut pending = vec![role];
let mut result = HashSet::new();
while let Some(next) = pending.pop() {
    if !seen.insert(next) { continue; }
    if let Some(direct) = grants.get(&next) { result.extend(direct.iter().copied()); }
    if let Some(inherited) = parents.get(&next) { pending.extend(inherited.iter().copied()); }
}
result
```

یک پشته‌ی صریح و یک مجموعه‌ی `seen`. `seen.insert` برایِ نقشِ دیده‌شده `false` برمی‌گرداند، و همین جلویِ حلقه را می‌گیرد: هر نقش یک بار پردازش می‌شود، پس حلقه بعد از حداکثر یک دیدار برایِ هر نقش تمام می‌شود.
