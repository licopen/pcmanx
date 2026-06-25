use pcmanx_rs::terminal::screen::ScreenBuffer;
use pcmanx_rs::terminal::vt100::process_bytes;

#[test]
fn put_char_writes_line_text() {
    let mut screen = ScreenBuffer::new(24, 80);
    screen.put_char('A');
    screen.put_char('B');
    screen.put_char('C');
    assert!(screen.get_line_text(0).starts_with("ABC"));
}

#[test]
fn clear_screen_sequence_works() {
    let mut screen = ScreenBuffer::new(24, 80);
    process_bytes(&mut screen, b"Hello");
    process_bytes(&mut screen, b"\x1b[2J");

    for r in 0..screen.rows {
        for c in 0..screen.cols {
            assert_eq!(screen.cell(r, c).ch, ' ');
        }
    }
}

#[test]
fn cursor_home_sequence_works() {
    let mut screen = ScreenBuffer::new(24, 80);
    process_bytes(&mut screen, b"\x1b[1;1H");
    assert_eq!(screen.cursor, (0, 0));
}

#[test]
fn sgr_green_sets_fg() {
    let mut screen = ScreenBuffer::new(24, 80);
    process_bytes(&mut screen, b"\x1b[32mHello\x1b[0m");
    assert_eq!(screen.cell(0, 0).ch, 'H');
    assert_eq!(screen.cell(0, 0).attr.fg, 2);
}

#[test]
fn carriage_return_overwrites_column_zero() {
    let mut screen = ScreenBuffer::new(24, 80);
    process_bytes(&mut screen, b"A\rB");
    assert_eq!(screen.cell(0, 0).ch, 'B');
}

#[test]
fn backspace_moves_cursor_back() {
    let mut screen = ScreenBuffer::new(24, 80);
    process_bytes(&mut screen, b"AB\x08C");
    assert_eq!(screen.cell(0, 1).ch, 'C');
}

#[test]
fn scroll_up_moves_rows() {
    let mut screen = ScreenBuffer::new(24, 80);
    for r in 0..24 {
        screen.go_to_xy(0, r);
        screen.put_char((b'A' + (r as u8 % 26)) as char);
    }

    let old_row1 = screen.get_line_text(1);
    screen.scroll_up(1);
    assert_eq!(screen.get_line_text(0), old_row1);
}

#[test]
fn sgr_bold_sets_bright() {
    let mut screen = ScreenBuffer::new(24, 80);
    process_bytes(&mut screen, b"\x1b[1mX");
    assert!(screen.cell(0, 0).attr.bright);
}

#[test]
fn sgr_inverse_sets_inverse() {
    let mut screen = ScreenBuffer::new(24, 80);
    process_bytes(&mut screen, b"\x1b[7mX");
    assert!(screen.cell(0, 0).attr.inverse);
}

#[test]
fn cursor_position_sequence() {
    let mut screen = ScreenBuffer::new(24, 80);
    process_bytes(&mut screen, b"\x1b[5;10H");
    assert_eq!(screen.cursor, (4, 9));
}
