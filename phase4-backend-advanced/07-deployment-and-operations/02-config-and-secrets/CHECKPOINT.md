# Checkpoint

1. Why does the real environment beat the `.env` file rather than the
   other way around? Describe a concrete production incident the reversed
   order would cause — think about what happens if a `.env` file
   accidentally ends up inside a container image.
2. `Config::resolve` runs once at startup and returns an error on the
   first missing required key. What's the argument for that over reading
   `std::env::var("DATABASE_URL")` lazily at the point of first use, and
   what must the error carry for the crash log to be actionable?
3. `Config` derives `Debug`, yet `format!("{:?}", config)` is safe to put
   in a log line. Walk through *why*: when the derived impl formats the
   `database_url` field, whose `Debug` implementation runs, and what
   would change if the field were a plain `String`?
4. `SecretString` can't prevent every leak. Name two ways the secret can
   still end up in a log or error message despite the redacting `Debug`
   impl — and why the loud, grep-able `expose()` name is the mitigation
   for exactly those.
5. `parse_env_file` splits each line on the *first* `=`. Give a realistic
   value that gets silently corrupted if you split on the last one (or on
   all of them) instead.
6. `defaults()` supplies `BIND_ADDR` and `LOG_LEVEL` but deliberately not
   `DATABASE_URL`. What are the two rules that decide whether a config
   key deserves a hardcoded default, and which one(s) does
   `DATABASE_URL` violate?
