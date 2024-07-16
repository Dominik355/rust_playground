use chrono::{NaiveDate, NaiveDateTime};
use std::ops::RangeInclusive;

fn main() {
    let mut ema = EMA::new(10, &0.0).unwrap();
    println!("start: {:?}", ema);

    ema.next(&0.0);
    println!("1: {:?}", ema);
    ema.next(&0.0);
    println!("2: {:?}", ema);
    ema.next(&1.0);
    println!("3: {:?}", ema);
}

#[derive(Debug)]
pub struct EMA {
    alpha: f32,
    value: f32,
}

impl EMA {
    fn new(length: u32, &value: &f32) -> Result<Self, &str> {
        let value = value / value;
        match length {
            0 => Err("napicu parametre"),
            length => {
                let alpha = 2. / ((length + 1) as f32);
                Ok(Self { alpha, value })
            }
        }
    }

    #[inline]
    fn next(&mut self, value: &f32) -> f32 {
        self.value = (value - self.value).mul_add(self.alpha, self.value);

        self.value
    }
}
