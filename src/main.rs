use std::io::Read;
const SCREEN_SIZE_FACTOR: usize = 1;

struct CPU {
    V: [u8; 0x10],
    memory: [u8; 0xfff],
    stack: Vec<u8>,
    PC: u16,
    I: u16,
    screen: [[u8; 64]; 32],
}

impl CPU {
    fn paint(&mut self) {
        //clear the screen with and set cursor to top left corner
        print!("\x1B[2J\x1B[1;1H");
        for (y, row) in self.screen.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                let p = self.screen[y / SCREEN_SIZE_FACTOR][x / SCREEN_SIZE_FACTOR];
                if p > 0 {
                    print!("█");
                } else {
                    print!(" ");
                }
            }
            print!("\n");
        }
    }
    fn read_instruction(&self) -> u16 {
        (self.memory[self.PC as usize] as u16) << 8
            | (self.memory[self.PC as usize + 1] as usize) as u16
    }

    fn step(&mut self) {
        let opCode = self.read_instruction();
        match opCode & 0xf000 {
            0x0000 => {
                match opCode & 0x0fff{
                    0x0e0=>{
                        self.clear_screen()
                    }
                }
            }
            0x1000 => {}
            0x2000 => {}
            0x3000 => {}
            0x4000 => {}
            0x5000 => {}
            0x6000 => {}
            0x7000 => {}
            0x8000 => {}
            0x9000 => {}
            0xa000 => {}
            0xb000 => {}
            0xc000 => {}
            0xd000 => {}
            0xe000 => {}
            0xf000 => {}
            _=>{}
        }
    }

    fn render_sprite(&mut self, start_x: u8, start_y: u8) {
        self.V[0xF] = 0;
        let startPosition = self.I as usize;
        for (i, y) in self.memory[startPosition..8].iter().enumerate() {
            let yy = (start_y + i as u8) as usize;
            for x in 0..7 {
                if isBitSet(*y, x) {
                    let xx = (x + start_x as u8) as usize;
                    self.screen[yy][xx] ^= 0xff;
                    if self.screen[yy][xx] == 0 {
                        self.V[0xF] = 1;
                    }
                }
            }
        }
    }
}

fn isBitSet(byte: u8, bit: u8) -> bool {
    return 1 & (byte >> bit) == 1;
}

fn printArray(arr: [u8; 0xfff]) {
    for (i, byte) in arr.iter().enumerate() {
        println!("{:>3X} {:0>8b}", i as u16, byte)
    }
}

fn main() {
    let mut cpu = CPU {
        memory: [0x0; 0xfff],
        stack: vec![],
        V: [0; 0x10],
        PC: 0x200,
        I: 0,
        screen: [[0x0; 64]; 32],
    };

    println!("{:X}", cpu.read_instruction());

    // let constructed_stdin = std::io::stdin();
    // let stdin_lock = constructed_stdin.lock();

    // for y in 0..32 {
    //     for x in 0..64 {
    //         cpu.screen[y][x] = if (y + x) % 2 == 0 { 0xff } else { 0x00 };
    //     }
    // }

    // cpu.paint();
    // let mut buffer:[u8;1] = [0; 1];
    // std::thread::sleep_ms(100);

    // // stdin_lock.read(&buffer);
    // for y in 0..32 {
    //     for x in 0..64 {
    //         cpu.screen[y][x] = if (y + x) % 2 == 0 { 0x00 } else { 0xff };
    //     }
    // }
    // cpu.paint();
    // std::thread::sleep_ms(100);

    // let fileResult = std::fs::read("./roms/demos/Maze (alt) [David Winter, 199x].ch8");
    // match fileResult {
    //     Ok(file) => {
    //         let end = 0x200 + file.len();
    //         cpu.memory.memory[0x200..end].copy_from_slice(&file);
    //         printArray(cpu.memory.memory);
    //     }
    //     Err(error) => panic!(error),
    // }
    // println!("{}", cpu.V[0xf]);
}
