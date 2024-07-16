fn main() {
    let mut a = vec![1, 2, 3];
    let mut x = 0;

    std::thread::scope(|scope| {
        scope.spawn(|| {
            println!("hello from the first scoped thread");
            // We can borrow `a` here.
            dbg!(&a);
        });
        scope.spawn(|| {
            println!("hello from the second scoped thread");
            // We can even mutably borrow `x` here,
            // because no other threads are using it.
            x += a[0] + a[2];
        });
        println!("hello from the main thread");
    });
    // scope is promised to end here, so we can be sure that threads
    // spawned within scope won't live longer than variables a, x

    // After the scope, we can modify and access our variables again:
    a.push(4);
    assert_eq!(x, a.len());
}
// fn main() {
//     let mut a = vec![1, 2, 3];
//     let mut x = 0;
//
//     std::thread::scope(|scope| {
//         scope.spawn(|| {
//             println!("hello from the first scoped thread");
//             // We can borrow `a` here.
//             dbg!(&a);
//         });
//         scope.spawn(|| {
//             println!("hello from the second scoped thread");
//             // We can even mutably borrow `x` here,
//             // because no other threads are using it.
//             x += a[0] + a[2];
//         });
//         println!("hello from the main thread");
//     });
//
//     // After the scope, we can modify and access our variables again:
//     a.push(4);
//     assert_eq!(x, a.len());
// }
