# ماژول ۳ — serialization و validation

extractor `Json<T>` در ماژول دو body را parse می‌کرد. این ماژول یک لایه پایین‌تر می‌رود: derive macroهای `serde` و مرحله‌ی validation صریح `validator`. در ماژول چهار endpoint واقعی create/update روی Postgres، همان الگوی دو مرحله‌ای آشنا خواهد بود.

۱. [`serde_json` و `validator`](01-serde-json-and-validator/README.fa.md): `Serialize`/`Deserialize`، attributeهای `#[serde(...)]` و ruleهای `#[derive(Validate)]`، جدا از HTTP.

سپس `CreateAnime` و `UpdateAnime` واقعی همان قواعد validator را می‌گیرند و persist می‌شوند.
