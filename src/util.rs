use std::cell::RefCell;
use std::thread::LocalKey;

pub trait WithInnerValue<T> {
  fn with_inner_value(&'static self, callback: impl Fn(&mut T));
}

impl<T> WithInnerValue<T> for LocalKey<RefCell<Option<T>>> {
  fn with_inner_value(&'static self, callback: impl Fn(&mut T)) {
    self.with(|rc| {
      let mut val_opt = rc.replace(None);
      match val_opt {
        Some(ref mut inner) => callback(inner),
        None => (),
      };
      rc.replace(val_opt);
    });
  }
}
