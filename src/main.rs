use crossterm::{
    ExecutableCommand, cursor,
    terminal::{self, Clear, ClearType},
};
use gilrs::{Axis, Gilrs};
use std::{
    io::{Write, stdout},
    thread,
    time::Duration,
};
use uart;

const BAR_WIDTH: usize = 40; // Szerokość paska w znakach

fn main() -> std::io::Result<()> {
    let mut gilrs = Gilrs::new().unwrap();
    let mut stdout = stdout();

    // Wyczyść ekran na początku
    stdout.execute(terminal::Clear(ClearType::All))?;
    stdout.execute(cursor::Hide)?; // Ukryj kursor dla estetyki

    println!("Oczekiwanie na dane z Radiomastera...");

    loop {
        // 1. Aktualizacja eventów (konieczne, aby biblioteka odświeżyła stan)
        while let Some(_) = gilrs.next_event() {}

        // 2. Pobierz pada (bierzemy pierwszy podłączony)
        if let Some((_id, gamepad)) = gilrs.gamepads().next() {
            // Pobieramy wartości osi (dostosuj osie do swojej konfiguracji Mode 1/2)
            // Zwykle Prawy Drążek to Ch1/Ch2 (Aileron/Elevator)
            let val_a = gamepad.value(Axis::LeftStickX);
            let val_b = gamepad.value(Axis::LeftStickY);

            // 3. Rysowanie interfejsu
            // Przesuwamy kursor na górę (zamiast czyścić wszystko, co by migało)
            stdout.execute(cursor::MoveTo(0, 0))?;

            println!("Urządzenie: {}\n", gamepad.name());

            // Rysujemy paski
            println!("Kanał A (X): {}", draw_bar(val_a));
            println!("Kanał B (Y): {}", draw_bar(val_b));

            // Wyświetl surowe wartości dla debugowania
            println!("\nRaw A: {:>5.8} | Raw B: {:>5.8}", val_a, val_b);

            let axis_a = (((val_a + 1.0) * 0.5 * (u32::MAX as f32)) as u32).to_be_bytes();
            let axis_b = (((val_b + 1.0) * 0.5 * (u32::MAX as f32)) as u32).to_be_bytes();

            dbg!(
                "Kanał a w u32: {:#x}, {:#x}, {:#x}, {:#x}",
                axis_a[0],
                axis_a[1],
                axis_a[2],
                axis_a[3]
            );
            dbg!(
                "Kanał b w u32: {:#x}, {:#x}, {:#x}, {:#x}",
                axis_b[0],
                axis_b[1],
                axis_b[2],
                axis_b[3]
            );

            let mut ramka = [0xff, 0xfe, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0fd];

            for (dst, src) in &mut ramka[2..]
                .iter_mut()
                .zip(axis_a.iter().chain(axis_b.iter()))
            {
                *dst = *src;
            }

            println!("{:02x?}", ramka);
        }

        // Flush konieczny przy manipulacji terminalem
        stdout.flush()?;

        // 60 FPS ;)
        thread::sleep(Duration::from_millis(16));
    }
}

// Funkcja pomocnicza do rysowania paska z zerem na środku
// Wygląd: [      |======   ]
fn draw_bar(value: f32) -> String {
    let empty_char = ' ';
    let fill_char = '=';
    let center_char = '|';

    let half_width = BAR_WIDTH / 2;
    let mut bar = String::with_capacity(BAR_WIDTH + 2);

    // Normalizacja wartości (czasem szum daje >1.0)
    let clamped_val = value.clamp(-1.0, 1.0);

    // Obliczamy ile znaków wypełnić
    // Wartość jest od -1 do 1, więc mnożymy przez połowę szerokości
    let fill_count = (clamped_val.abs() * half_width as f32).round() as usize;

    bar.push('[');

    if clamped_val < 0.0 {
        // Wychylenie w lewo: [   ====|      ]
        let spaces = half_width.saturating_sub(fill_count);
        for _ in 0..spaces {
            bar.push(empty_char);
        }
        for _ in 0..fill_count {
            bar.push(fill_char);
        }
        bar.push(center_char);
        for _ in 0..half_width {
            bar.push(empty_char);
        }
    } else {
        // Wychylenie w prawo: [      |====   ]
        for _ in 0..half_width {
            bar.push(empty_char);
        }
        bar.push(center_char);
        for _ in 0..fill_count {
            bar.push(fill_char);
        }
        let spaces = half_width.saturating_sub(fill_count);
        for _ in 0..spaces {
            bar.push(empty_char);
        }
    }

    bar.push(']');
    bar
}
