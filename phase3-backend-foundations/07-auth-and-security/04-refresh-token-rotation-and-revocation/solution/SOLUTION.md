# Solution — 3.7.4 Refresh-token rotation and revocation

## `hash_token`

```rust
use sha2::{Digest, Sha256};

pub fn hash_token(raw: &str) -> String {
    format!("{:x}", Sha256::digest(raw.as_bytes()))
}
```

`Sha256::digest` returns the 32 raw bytes. The `{:x}` format of that array prints lowercase hex, two characters per byte, so 64 in all. The `Digest` trait has to be in scope for `digest` to be callable, which is why the skeleton asks you to add the `use`.

## `rotate`

```rust
let hash = hash_token(presented);
let record = self.records.get(&hash).cloned().ok_or(RefreshError::Unknown)?;
if record.used {
    self.revoke_family(&record.family_id);
    return Err(RefreshError::ReuseDetected);
}
if record.expires_at <= self.clock.now() {
    return Err(RefreshError::Expired);
}
if let Some(r) = self.records.get_mut(&hash) {
    r.used = true;
}
Ok(self.issue(&record.user_id, &record.family_id))
```

The order is the specification: unknown, then used, then expired. The record is cloned out of the map so the later `&mut self` calls (`revoke_family`, `issue`) do not fight a live borrow of `self.records`. `expires_at <= now` makes a token expire exactly at its expiry second. The new token goes into the same family, which is what lets one replay revoke the whole chain.

## `logout`

```rust
let Some(record) = self.records.get(&hash_token(presented)).cloned() else {
    return false;
};
self.revoke_family(&record.family_id);
true
```

No `used` or expiry check: a user who logs out with an old or expired token still means "end this session".

## `logout_all`

```rust
let mut families: Vec<String> = self
    .records
    .values()
    .filter(|r| r.user_id == user_id)
    .map(|r| r.family_id.clone())
    .collect();
families.sort();
families.dedup();
for family in &families {
    self.revoke_family(family);
}
families.len()
```

A family has one record per rotation, so the family ids repeat; sorting and deduplicating counts each family once. Collecting the ids first ends the borrow of `self.records` before `revoke_family` needs `&mut self`.

## The challenge

Add `used_at: Option<u64>` to `Record`. In `rotate`, when the record is used and `now - used_at <= GRACE_SECONDS`, return `Err(RefreshError::Unknown)` without calling `revoke_family`; outside the window, revoke as before. Returning a refusal and not the same new pair keeps the rule "a token yields at most one new pair". Test it with a `ManualClock` at 9, 10 and 11 seconds after the rotation.
