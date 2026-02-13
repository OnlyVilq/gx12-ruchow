use crossterm::{
    cursor,
    terminal::{self, ClearType}, // Usunięto zbędne 'Clear'
    ExecutableCommand,
};
use gilrs::{Axis, Gilrs};
use std::{
    io::{stdout, Write},
    thread,
    time::Duration,
};

const BAR_WIDTH: usize = 40;
const PORT_NAME: &str = "/dev/ttyACM1";
const BAUD_RATE: u32 = 115_200;

// Zmiana Result<()> na Result<(), Box<dyn ...>> pozwala obsługiwać błędy z różnych bibliotek
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Obsługa błędu Gilrs przy inicjalizacji
    let mut gilrs = Gilrs::new().map_err(|e| format!("Nie udało się zainicjować Gilrs: {}", e))?;
    let mut stdout = stdout();

    println!("Otwieranie portu {}...", PORT_NAME);
    
    // Otwarcie portu (unwrap/expect rzuci panic, jeśli się nie uda, co jest OK na start)
    let mut port = serialport::new(PORT_NAME, BAUD_RATE)
        .timeout(Duration::from_millis(100))
        .open()?; 

    stdout.execute(terminal::Clear(ClearType::All))?;
    stdout.execute(cursor::Hide)?;

    loop {
        while let Some(_) = gilrs.next_event() {}

        if let Some((_id, gamepad)) = gilrs.gamepads().next() {
            let val_a = gamepad.value(Axis::LeftStickX);
            let val_b = gamepad.value(Axis::LeftStickY);

            stdout.execute(cursor::MoveTo(0, 0))?;
            println!("Urządzenie: {} ({})\n", gamepad.name(), PORT_NAME);

            println!("Kanał A (X): {}", draw_bar(val_a));
            println!("Kanał B (Y): {}", draw_bar(val_b));
            println!("\nRaw A: {:>5.8} | Raw B: {:>5.8}   ", val_a, val_b);

            let axis_a = (((val_a + 1.0) * 0.5 * (u32::MAX as f32)) as u32).to_be_bytes();
            let axis_b = (((val_b + 1.0) * 0.5 * (u32::MAX as f32)) as u32).to_be_bytes();

            let mut ramka = [0xff, 0xfe, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xfd];

            for (dst, src) in ramka[2..].iter_mut().zip(axis_a.iter().chain(axis_b.iter())) {
                *dst = *src;
            }

            // Próba wysłania danych
            if let Err(e) = port.write_all(&ramka) {
                // Jeśli padnie UART, wychodzimy z pętli
                // stdout flush jest niżej, ale warto przywrócić kursor przed wyjściem
                stdout.execute(cursor::Show)?;
                eprintln!("\n\nBłąd zapisu UART: {}", e);
                break; 
            }
        } else {
            stdout.execute(cursor::MoveTo(0, 0))?;
            println!("Oczekiwanie na pada...                    ");
        }

        stdout.flush()?;
        thread::sleep(Duration::from_millis(16));
    }

    // To jest kluczowe - po wyjściu z pętli (break) zwracamy Ok(())
    Ok(())
}

fn draw_bar(value: f32) -> String {
    let empty_char = ' ';
    let fill_char = '=';
    let center_char = '|';

    let half_width = BAR_WIDTH / 2;
    let mut bar = String::with_capacity(BAR_WIDTH + 2);

    let clamped_val = value.clamp(-1.0, 1.0);
    let fill_count = (clamped_val.abs() * half_width as f32).round() as usize;

    bar.push('[');

    if clamped_val < 0.0 {
        let spaces = half_width.saturating_sub(fill_count);
        for _ in 0..spaces { bar.push(empty_char); }
        for _ in 0..fill_count { bar.push(fill_char); }
        bar.push(center_char);
        for _ in 0..half_width { bar.push(empty_char); }
    } else {
        for _ in 0..half_width { bar.push(empty_char); }
        bar.push(center_char);
        for _ in 0..fill_count { bar.push(fill_char); }
        let spaces = half_width.saturating_sub(fill_count);
        for _ in 0..spaces { bar.push(empty_char); }
    }

    bar.push(']');
    bar
}