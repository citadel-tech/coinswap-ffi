Important Caveat :

JS doesnt have enums the way Rust has. Napi converts Enums into String Objects for JS.

```
#[napi(string_enum)]
pub enum LockTimeNapi {
  Blocks,
  Seconds,
}

impl From<bitcoin::absolute::LockTime> for LockTimeNapi {
  fn from(locktime: bitcoin::absolute::LockTime) -> Self {
    match locktime {
      bitcoin::absolute::LockTime::Blocks(_) => LockTimeNapi::Blocks,
      bitcoin::absolute::LockTime::Seconds(_) => LockTimeNapi::Seconds,
    }
  }
}
```
becomes 

```
export const enum LockTimeNapi {
  Blocks = 'Blocks',
  Seconds = 'Seconds',
}
```