# ایست بازرسی

۱. اگر assertion را حذف و `mid = 10` را برای slice سه‌عضوی بدهی، `ptr.add(mid)` چه می‌کند؟ خود ساخت pointer خارج allocation چه محدودیتی دارد و dereference آن چه خطری؟
۲. کامنت `SAFETY` دقیقاً چه ادعایی می‌کند و حفظ آن مسئولیت compiler است یا تو؟ تغییر اشتباه assertion چه اثری دارد؟
۳. پنج توانایی `unsafe` را نام ببر و بگو تمرین از کدام استفاده می‌کند.
۴. چرا borrow checker non-overlapبودن `&mut slice[..mid]` و `&mut slice[mid..]` را از indexing پویا اثبات نمی‌کند؟
