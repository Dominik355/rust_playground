fn main() {
    let vec = vec![0, 1, 2, 3, 4, 5, 6, 7];
    for i in 1..=8 {
        println!("split_vec: {:?}", split_vec(vec.clone(), i));
    }
}

fn split_vec<T>(mut vec: Vec<T>, mut parts: usize) -> Vec<Vec<T>> {
    assert!(parts > 0);
    parts = std::cmp::min(vec.len(), parts);
    let len = vec.len();
    let mut result = Vec::with_capacity(parts);

    for i in 0..parts {
        let part_size = if i < len % parts {
            (len / parts) + 1
        } else {
            len / parts
        };
        result.push(vec.drain(0..part_size).collect());
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_vec_test() {
        assert_eq!(
            split_vec(vec![0, 1, 2, 3, 4, 5, 6, 7], 1),
            vec![vec![0, 1, 2, 3, 4, 5, 6, 7]]
        );
        assert_eq!(
            split_vec(vec![0, 1, 2, 3, 4, 5, 6, 7], 2),
            vec![vec![0, 1, 2, 3], vec![4, 5, 6, 7]]
        );
        assert_eq!(
            split_vec(vec![0, 1, 2, 3, 4, 5, 6, 7], 3),
            vec![vec![0, 1, 2], vec![3, 4, 5], vec![6, 7]]
        );
        assert_eq!(
            split_vec(vec![0, 1, 2, 3, 4, 5, 6, 7], 4),
            vec![vec![0, 1], vec![2, 3], vec![4, 5], vec![6, 7]]
        );
        assert_eq!(
            split_vec(vec![0, 1, 2, 3, 4, 5, 6, 7], 5),
            vec![vec![0, 1], vec![2, 3], vec![4, 5], vec![6], vec![7]]
        );
        assert_eq!(
            split_vec(vec![0, 1, 2, 3, 4, 5, 6, 7], 6),
            vec![vec![0, 1], vec![2, 3], vec![4], vec![5], vec![6], vec![7]]
        );
        assert_eq!(
            split_vec(vec![0, 1, 2, 3, 4, 5, 6, 7], 7),
            vec![
                vec![0, 1],
                vec![2],
                vec![3],
                vec![4],
                vec![5],
                vec![6],
                vec![7]
            ]
        );
        assert_eq!(
            split_vec(vec![0, 1, 2, 3, 4, 5, 6, 7], 8),
            vec![
                vec![0],
                vec![1],
                vec![2],
                vec![3],
                vec![4],
                vec![5],
                vec![6],
                vec![7]
            ]
        );
    }
}

// use std::future::Future;
// use std::pin::Pin;
// use std::time::Duration;
// use tokio::time::{Instant, Sleep};
//
// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let mut rate_limit = RateLimit::new(Rate::new(2, Duration::from_secs(1)));
//
//     loop {
//         println!(
//             "{:?} | May I ? {}",
//             chrono::Local::now(),
//             rate_limit.may_i()
//         );
//         tokio::time::sleep(Duration::from_millis(231)).await;
//     }
//
//     Ok(())
// }
//
// #[derive(Debug)]
// pub struct RateLimit {
//     rate: Rate,
//     state: State,
//     sleep: Pin<Box<Sleep>>,
// }
//
// impl RateLimit {
//     /// Create a new rate limiter
//     pub fn new(rate: Rate) -> Self {
//         let until = Instant::now();
//         let state = State::Ready {
//             until,
//             rem: rate.num(),
//         };
//
//         RateLimit {
//             rate,
//             state,
//             // The sleep won't actually be used with this duration, but
//             // we create it eagerly so that we can reset it in place rather than
//             // `Box::pin`ning a new `Sleep` every time we need one.
//             sleep: Box::pin(tokio::time::sleep_until(until)),
//         }
//     }
//
//     fn may_i(&mut self) -> bool {
//         match self.state {
//             State::Ready { mut until, mut rem } => {
//                 let now = Instant::now();
//
//                 // If the period has elapsed, reset it.
//                 if now >= until {
//                     until = now + self.rate.per();
//                     rem = self.rate.num();
//                 }
//
//                 if rem > 1 {
//                     rem -= 1;
//                     self.state = State::Ready { until, rem };
//                 } else {
//                     // The service is disabled until further notice
//                     // Reset the sleep future in place, so that we don't have to
//                     // deallocate the existing box and allocate a new one.
//                     self.sleep.as_mut().reset(until);
//                     self.state = State::Limited;
//                 }
//                 true
//             }
//             State::Limited => {
//                 if Pin::new(&mut self.sleep).poll(cx).is_pending() {
//                     tracing::trace!("rate limit exceeded; sleeping.");
//                     return Poll::Pending;
//                 }
//
//                 self.state = State::Ready {
//                     until: Instant::now() + self.rate.per(),
//                     rem: self.rate.num(),
//                 };
//             }
//         }
//     }
// }
//
// #[derive(Debug)]
// enum State {
//     // Limit has been hit
//     Limited,
//     Ready { until: Instant, rem: u64 },
// }
//
// /// A rate of something per time period.
// #[derive(Debug, Copy, Clone)]
// pub struct Rate {
//     num: u64,
//     per: Duration,
// }
//
// impl Rate {
//     pub fn new(num: u64, per: Duration) -> Self {
//         assert!(num > 0);
//         assert!(per > Duration::from_millis(0));
//
//         Rate { num, per }
//     }
//
//     pub(crate) fn num(&self) -> u64 {
//         self.num
//     }
//
//     pub(crate) fn per(&self) -> Duration {
//         self.per
//     }
// }
