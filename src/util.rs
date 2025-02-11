

pub fn as_decimal(value: i32, decimals: usize) -> String{
    // let f = 10 ^ decimals as u32;
    let w = if value < 0 { decimals + 2} else {decimals + 1};
    let mut s = format!("{:0width$}", value, width = w);

    let mut l = s.len();

    if l <= decimals {
      while l < decimals{
        s.insert(0, '0');
        l += 1;
      }
      s.insert(0, '.');
      s.insert(0, '0');
    }
    else {
      s.insert(s.len() - decimals, '.');
    }
    // println!("as_decimal({}, {}) = {}", value, decimals, s);
    s
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_int() {
         assert_eq!(as_decimal(36, 2), "0.36");
         assert_eq!(as_decimal(-36, 2), "-0.36");
    }
}