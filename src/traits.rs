use std::fmt::Display;

pub trait Commas {
    fn commas(&self) -> String;
}

impl<T: Display> Commas for T {
    fn commas(&self) -> String {
        let string = self.to_string();
        let (integer, fraction) = string.split_once('.').unwrap_or((&string, ""));
        let mut formatted_integer = String::new();

        for (index, char) in integer.chars().enumerate() {
            if (integer.len() - index) % 3 == 0 && index != 0 {
                formatted_integer += ",";
            }

            formatted_integer += &char.to_string();
        }

        if fraction.is_empty() {
            formatted_integer
        } else {
            format!("{formatted_integer}.{fraction}")
        }
    }
}
