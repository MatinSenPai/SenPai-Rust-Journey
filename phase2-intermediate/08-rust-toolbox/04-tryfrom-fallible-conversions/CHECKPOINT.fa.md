# ایست بازرسی

۱. قاعده‌ی انتخاب `From` و `TryFrom` چیست و چرا `From` دارای panic دروغ نوعی است؟
۲. با اینکه فقط `TryFrom<u8> for Percentage` نوشته‌ای، `42u8.try_into()` از کجا می‌آید؟
۳. `300u64 as u8` و `u8::try_from(300u64)` دقیقاً چه نتایجی دارند و چرا اولی ۴۴ می‌شود؟
۴. publicکردن field داخلی `Percentage` کدام تضمین را از بین می‌برد؟
۵. `is_valid_email(&str) -> bool` با `TryFrom<String> for EmailAddress` چه تفاوتی در promise امضای downstream دارد؟
۶. نگهداری `String` ردشده در `ValidationError::InvalidEmail` چه هزینه و چه فایده‌ای دارد؟
