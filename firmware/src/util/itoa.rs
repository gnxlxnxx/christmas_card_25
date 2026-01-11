pub fn utoa10(mut val: u32, str: &mut [u8; 10]) -> &mut [u8] {
    let i = str
        .iter_mut()
        .enumerate()
        .find_map(|(i, c)| {
            *c = b'0' + (val % 10) as u8;
            val /= 10;

            if val == 0 { Some(i) } else { None }
        })
        .unwrap();

    let str = &mut str[0..=i];
    str.reverse();

    str
}
