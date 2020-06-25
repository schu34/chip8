use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;

const SCREEN_SIZE_FACTOR: usize = 1;

struct CPU {
    v: [u8; 0x10],
    memory: [u8; 0xfff],
    stack: [u16; 0x10],
    pc: u16,
    sp: u16,
    i: u16,
    screen: [[u8; 64]; 32],
    dt: u8,
    st: u8,
    key: [bool; 0x10],
}

const FONT_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl CPU {
    fn paint(&mut self, canvas: &mut Canvas<sdl2::video::Window>) {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.set_draw_color(Color::WHITE);
        for (y, row) in self.screen.iter().enumerate() {
            for (x, _) in row.iter().enumerate() {
                let p = self.screen[y / SCREEN_SIZE_FACTOR][x / SCREEN_SIZE_FACTOR];
                if p > 0 {
                    let result = canvas.fill_rect(Rect::new((x as i32) * 10, (y as i32) * 10, 10, 10));
                    match result{
                        Ok(_)=>{},
                        Err(e)=>{panic!(e)}
                    }
                }
            }
        }
        canvas.present();
    }
    fn read_instruction(&self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[self.pc as usize + 1] as usize) as u16
    }

    fn clear_screen(&mut self) {
        self.screen = [[0; 64]; 32];
        self.pc += 2;
    }
    fn return_op(&mut self) {
        let sp = self.sp as usize;
        self.pc = self.stack[sp - 1];
        self.sp -= 1;
    }
    fn jump(&mut self, address: u16) {
        self.pc = address;
    }
    fn call(&mut self, nnn: u16) {
        println!("call, {:X}", nnn);
        let sp = self.sp as usize;
        self.stack[sp] = self.pc + 2;
        self.sp += 1;
        self.pc = nnn;
    }
    fn check_equality(&mut self, x: usize, nn: u8) {
        self.pc += 2;
        if self.v[x] == nn {
            self.pc += 2;
        }
    }
    fn check_inequality(&mut self, x: usize, nn: u8) {
        self.pc += 2;
        if self.v[x] != nn {
            self.pc += 2;
        }
    }
    fn check_v_equality(&mut self, x: usize, y: usize) {
        self.pc += 2;
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }
    fn check_v_inequality(&mut self, x: usize, y: usize) {
        self.pc += 2;
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }
    fn set_register(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
        self.pc += 2;
    }
    fn add_to_register(&mut self, x: usize, nn: u8) {
        let (result, _did_overflow) = self.v[x].overflowing_add(nn);
        self.v[x] = result;
        self.pc += 2;
    }
    fn assign_register(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
        self.pc += 2;
    }
    fn assign_register_or(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
        self.pc += 2;
    }
    fn assign_register_and(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
        self.pc += 2;
    }
    fn assign_register_xor(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
        self.pc += 2;
    }
    fn assign_register_add(&mut self, x: usize, y: usize) {
        let (result, did_overflow) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = result;
        if did_overflow {
            self.v[0xf] = 1
        } else {
            self.v[0xf] = 0
        };
        self.pc += 2;
    }
    fn assign_register_sub(&mut self, x: usize, y: usize) {
        let (result, did_overflow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = result;
        if did_overflow {
            self.v[0xf] = 1
        } else {
            self.v[0xf] = 0
        };
        self.pc += 2;
    }
    fn shift_register_right(&mut self, x: usize) {
        self.v[0xf] = self.v[x] & 0b1;
        self.v[x] >>= 1;
        self.pc += 2
    }
    fn shift_register_left(&mut self, x: usize) {
        self.v[0xf] = (self.v[x] & 0b10000000) >> 7;
        self.v[x] <<= 1;
        self.pc += 2
    }
    fn assign_register_sub_reversed(&mut self, x: usize, y: usize) {
        let (result, did_overflow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = result;
        if did_overflow {
            self.v[0xf] = 1
        } else {
            self.v[0xf] = 0
        };
        self.pc += 2;
    }
    fn set_i(&mut self, nnn: u16) {
        self.i = nnn;
        self.pc += 2
    }
    fn jump_plus_v0(&mut self, nnn: u16) {
        self.pc = nnn + self.v[0] as u16;
    }
    fn and_rand(&mut self, x: usize, nn: u8) {
        let mut rng = rand::thread_rng();
        let random_number: u8 = rng.gen();
        self.v[x] = nn & random_number;
        self.pc += 2;
    }
    fn reg_dump(&mut self, x: usize) {
        let i_ptr = self.i as usize;
        for offset in 0..=x {
            self.memory[i_ptr + offset] = self.v[offset];
        }
        self.pc += 2;
    }
    fn reg_load(&mut self, x: usize) {
        let i_ptr = self.i as usize;
        for offset in 0..=x {
            self.v[offset] = self.memory[i_ptr + offset];
        }
        self.pc += 2;
    }

    fn to_bcd(&mut self, x: usize) {
        let mut value = self.v[x].clone();
        for i in 0..3 {
            let offset = 2 - i;
            let digit: u8 = value % 10;
            self.memory[self.i as usize + offset] = digit;
            value /= 10;
        }
        self.pc += 2;
    }
    fn set_delay_timer(&mut self, x: usize) {
        self.dt = self.v[x];
        self.pc += 2;
    }
    fn set_sound_timer(&mut self, x: usize) {
        self.st = self.v[x];
        self.pc += 2;
    }
    fn get_delay(&mut self, x: usize) {
        self.v[x] = self.dt;
        self.pc += 2;
    }
    fn add_to_i(&mut self, x: usize) {
        self.i += self.v[x] as u16;
        self.pc += 2;
    }
    fn set_i_to_font_ptr(&mut self, x: usize) {
        let ch = self.v[x];
        let addr = ch * 5;
        self.i = addr as u16;
        self.pc += 2;
    }
    fn is_key_down(&mut self, x: usize){
        let key = self.v[x];
        if self.key[key as usize]{
            self.pc+=2
        }
        self.pc+=2
    }
    fn is_key_up(&mut self, x: usize){
        let key = self.v[x];
        if !self.key[key as usize]{
            self.pc+=2
        }
        self.pc+=2
    }

    fn wait_for_key(&mut self, x:usize){
        for i in 0..16{
            if self.key[i] {
                self.v[x]= self.v[i];
                self.pc += 2
            }
        }
    }

    fn step(&mut self) {
        let op_code = self.read_instruction();
        let x = ((op_code >> 8) & 0x000f) as usize;
        let y = ((op_code >> 4) & 0x000f) as usize;
        let n = (op_code & 0x000f) as u8;
        let nn = (op_code & 0x00ff) as u8;
        let nnn = op_code & 0x0fff;
        println!("op: {:X}, {:X}", ( op_code & 0xf000 ), op_code);
        if self.st > 0 {
            self.st -= 1
        };
        if self.dt > 0 {
            self.dt -= 1
        };
        match op_code & 0xf000  {
            0x0000 => match op_code & 0x0fff {
                0x0e0 => self.clear_screen(),
                0x0ee => self.return_op(),
                0x000 => panic!("null instruction found, quitting"),
                _ => {}
            },
            0x1000 => self.jump(nnn),
            0x2000 => self.call(nnn),
            0x3000 => self.check_equality(x, nn),
            0x4000 => self.check_inequality(x, nn),
            0x5000 => self.check_v_equality(x, y),
            0x6000 => self.set_register(x, nn),
            0x7000 => self.add_to_register(x, nn),
            0x8000 => match n {
                0x0 => self.assign_register(x, y),
                0x1 => self.assign_register_or(x, y),
                0x2 => self.assign_register_and(x, y),
                0x3 => self.assign_register_xor(x, y),
                0x4 => self.assign_register_add(x, y),
                0x5 => self.assign_register_sub(x, y),
                0x6 => self.shift_register_right(x),
                0x7 => self.assign_register_sub_reversed(x, y),
                0xe => self.shift_register_left(x),
                _ => {}
            },
            0x9000 => self.check_v_inequality(x, y),
            0xa000 => self.set_i(nnn),
            0xb000 => self.jump_plus_v0(nnn),
            0xc000 => self.and_rand(x, nn),
            0xd000 => self.render_sprite(x, y, n),
            0xe000 => match nn{
                0x9e=>self.is_key_down(x),
                0xa1=>self.is_key_up(x),
                _=>{}
            }
            0xf000 => match nn {
                0x07 => self.get_delay(x),
                0x0A => self.wait_for_key(x),
                0x15 => self.set_delay_timer(x),
                0x18 => self.set_sound_timer(x),
                0x1e => self.add_to_i(x),
                0x29 => self.set_i_to_font_ptr(x),
                0x33 => self.to_bcd(x),
                0x55 => self.reg_dump(x),
                0x65 => self.reg_load(x),
                _ => {}
            },
            _ => {}
        }
    }
    fn render_sprite(&mut self, start_x_reg: usize, start_y_reg: usize, height: u8) {
        self.v[0xf] = 0;
        let start_position = self.i as usize;
        let start_y = self.v[start_y_reg];
        let start_x = self.v[start_x_reg];
        for (y, row) in self.memory[start_position..(start_position + height as usize)]
            .iter()
            .enumerate()
        {
            let yy = (start_y + y as u8) as usize;
            for x in 0..8 {
                if x + start_x > 63 {continue}
                if is_bit_set(*row, 7 - x) {
                    let xx = (x + start_x as u8) as usize;
                    self.screen[yy][xx] ^= 0xff;
                    if self.screen[yy][xx] == 0 {
                        self.v[0xF] = 1;
                    }
                }
            }
        }
        self.pc += 2;
    }
}

fn is_bit_set(byte: u8, bit: u8) -> bool {
    return 1 & (byte >> bit) == 1;
}

fn print_memory(arr: [u8; 0xfff], len: usize) {
    for (i, byte) in arr[0x200..0x200 + len].iter().enumerate() {
        println!("{:>3X} {1:0>8b} {1:0>2X}", i as u16, byte)
    }
}

fn print_cpu_state(cpu: &CPU) {
    println!(
        "PC: {:0<3X} SP: {:<3X} I: {:0<3X} Next: {:0>4X}",
        cpu.pc,
        cpu.sp,
        cpu.i,
        cpu.read_instruction()
    );
    println!("|index|Stack|  V  | key |");
    println!("--------------------");
    for i in 0..0x10 {
        println!("|{:<5X}|{:<5X}|{:<5X}|{:<5}|", i, cpu.stack[i], cpu.v[i], cpu.key[i]);
    }
    println!("--------------------\n\n");
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("chip8", 640, 320)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut cpu = CPU {
        memory: [0x0; 0xfff],
        stack: [0; 0x10],
        v: [0; 0x10],
        pc: 0x200,
        i: 0,
        sp: 0,
        screen: [[0x0; 64]; 32],
        st: 0,
        dt: 0,
        key: [false; 0x10],
    };
    cpu.memory[0..80].copy_from_slice(&FONT_DATA);

    let file_result =
        std::fs::read("roms/programs/Fishie [Hap, 2005].ch8");
    match file_result {
        Ok(file) => {
            let len = file.len();
            let end = 0x200 + len;
            cpu.memory[0x200..end].copy_from_slice(&file);
            print_memory(cpu.memory, len);
        }
        Err(error) => panic!(error),
    }
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        cpu.paint(&mut canvas);
        print_cpu_state(&cpu);
        cpu.step();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::Escape) => break 'running,
                    Some(Keycode::Up) => cpu.key[0x2] = true,
                    Some(Keycode::Left) => cpu.key[0x4] = true,
                    Some(Keycode::Right) => cpu.key[0x6] = true,
                    Some(Keycode::Down) => cpu.key[0x8] = true,
                    _ => {}
                },
                Event::KeyUp { keycode, .. } => match keycode {
                    Some(Keycode::Up) => cpu.key[0x2] = false,
                    Some(Keycode::Left) => cpu.key[0x4] = false,
                    Some(Keycode::Right) => cpu.key[0x6] = false,
                    Some(Keycode::Down) => cpu.key[0x8] = false,
                    _ => {}
                },
                _ => {}
            }
        }
        // std::thread::sleep_ms(1000 / 60);
    }

}
