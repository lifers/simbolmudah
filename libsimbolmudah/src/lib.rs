#[no_mangle]
pub fn hello(name: &str) -> String {
  format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let result = hello("Robert");
    assert_eq!(result, "Hello, Robert!")
  }
}