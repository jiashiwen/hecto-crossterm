use unicode_segmentation::UnicodeSegmentation;

use unicode_width::UnicodeWidthChar;

fn main() {
    let mut v = vec![1, 2, 3];
    v.insert(1, 9);
    let s = "abc,中国";
    let w = s.graphemes(true).collect::<Vec<&str>>();
    println!("vec is {:?}", v);

    println!("{},{}", w.len(), s.len());

    for c in s.chars() {
        println!("{:?}", unicode_width::UnicodeWidthChar::width(c));
    }
}