use std::env;
use std::ffi::OsStr;
use std::ops::Drop;

pub fn set_if_unset<K, V>(k: K, v: V) -> TmpEnv<K>
    where K: AsRef<OsStr>,
          V: AsRef<OsStr>,
{
    match env::var(&k) {
        Ok(_) => TmpEnv::WasAlreadySet,
        Err(_) => {
            env::set_var(&k, v);
            TmpEnv::WasNotSet { k: k }
        }
    }
}

#[must_use]
pub enum TmpEnv<K>
    where K: AsRef<OsStr>,
{
    WasAlreadySet,
    WasNotSet {
        k: K,
    }
}

impl<K> Drop for TmpEnv<K>
    where K: AsRef<OsStr>,
{
    fn drop(&mut self) {
        if let TmpEnv::WasNotSet { ref k } = *self {
            env::remove_var(k);
        }
    }
}
