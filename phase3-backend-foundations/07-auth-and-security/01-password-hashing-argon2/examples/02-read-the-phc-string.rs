//! Take a stored hash apart, field by field.
//! Run: `cargo run -p p3-07-01-password-hashing-argon2 --example 02-read-the-phc-string`

use argon2::password_hash::PasswordHash;
use argon2::Params;

fn main() {
    let phc = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG";
    let parsed = PasswordHash::new(phc).expect("not a PHC string");
    let params = Params::try_from(&parsed).expect("bad params");

    println!("algorithm  : {}", parsed.algorithm);
    println!("version    : {:?}", parsed.version);
    println!("memory KiB : {}", params.m_cost());
    println!("iterations : {}", params.t_cost());
    println!("lanes      : {}", params.p_cost());
    println!("salt       : {}", parsed.salt.expect("no salt"));
    println!("hash bytes : {}", parsed.hash.expect("no hash").len());
}
