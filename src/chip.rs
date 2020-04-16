use rand::Rng;
use console::*;

use std::fs::File;
use std::io::Read;

const FONT: [[u8; 5]; 16]= [
    [0xf0, 0x90, 0x90, 0x90, 0xf0], //0
    [0x20, 0x60, 0x20, 0x20, 0x70], //1
    [0xf0, 0x10, 0xf0, 0x80, 0xf0], //2
    [0xf0, 0x10, 0xf0, 0x10, 0xf0], //3
    [0x90, 0x90, 0xf0, 0x10, 0x10], //4
    [0xf0, 0x80, 0xf0, 0x10, 0xf0], //5
    [0xf0, 0x80, 0xf0, 0x90, 0xf0], //6
    [0xf0, 0x10, 0x20, 0x40, 0x40], //7
    [0xf0, 0x90, 0xf0, 0x90, 0xf0], //8
    [0xf0, 0x90, 0xf0, 0x10, 0xf0], //9
    [0xf0, 0x90, 0xf0, 0x90, 0x90], //a
    [0xe0, 0x90, 0xe0, 0x90, 0xe0], //b
    [0xf0, 0x80, 0x80, 0x80, 0xf0], //c
    [0xe0, 0x90, 0x90, 0x90, 0xe0], //d
    [0xf0, 0x80, 0xf0, 0x80, 0xf0], //e
    [0xf0, 0x80, 0xf0, 0x80, 0x80], //0
];

pub struct Registers {
    gpr: [u8; 16], //general purpose registers
    i: u16,        //I register, usually used to store memory addresses
    dt: u8,     //for st timers
    st: u8,     //for st timers
    pc: u16,       //program counter
    sp: i8,        //stack pointer
}

impl Registers {
    pub fn new()-> Registers {
        Registers {
            gpr: [0; 16],
            i: 0,
            dt: 0,
            st: 0,
            pc: 0x200,
            sp: -1,
        }
    }
}

pub struct Memory {
    data: [u8; 4096],
    stack: [u16; 16],
}

impl Memory {
    pub fn new(program: &str)-> Memory {
        let mut m = Memory {
            data: [0; 4096],
            stack: [0; 16],
        };

        let mut file=File::open(program).unwrap();
        let mut buf = [0u8];
        
        let mut index = 0x200;
        while file.read(&mut buf).unwrap() != 0 {
            m.data[index] = buf[0];
            index += 1;
        }

        for i in 0..16 {
            for j in 0..5 {
                m.data[i * 5 + j] = FONT[i][j];
            }
        }
        m
    }
}

pub struct Display {
    pub pixels: [u64; 32],
}

fn right_rotate(n: u64, s: u8)-> u64 {
    let shift;

    if s == 0 {
        shift = 0;
    } else {
        shift = 64 - s;
    }

    (n >> s) | (n << shift)
}

impl Display {
    pub fn new()-> Display {
        Display {
            pixels: [0; 32],
        }
    }

    pub fn clear(&mut self) {
        for i in 0..32 {
            self.pixels[i] = 0;
        }
    }

    pub fn draw_sprite(&mut self, sprite: [u8; 15], n: u16, (x, y): (u8, u8))-> bool {
        let mut big_sprite: [u64; 15] = [0; 15]; 
        
        for i in 0..15 {
            big_sprite[i] = (sprite[i] as u64) << 56;
        }

        for i in 0..n {
            let y_index = (y as u16 + i) % 32;
            self.pixels[y_index as usize] = self.pixels[y_index as usize] ^ right_rotate(big_sprite[i as usize], x);
        }
        
        true
    }

    pub fn print(&self) {
        for row in self.pixels.iter() {
            println!("{:#066b}", row);
        }
    }
}

pub struct Chip {
    regs: Registers,
    mem: Memory,
    pub display: Display,
}

fn get_vx_k(opcode: u16)-> (usize, u8) {
    let vx = ((opcode & 0x0f00) >> 8) as usize;
    let k = (opcode & 0x00ff) as u8;
    (vx, k)
}

fn get_vx_vy(opcode: u16)-> (usize, usize) {
    let vx = ((opcode & 0x0f00) >> 8) as usize;
    let vy = ((opcode & 0x00f0) >> 4) as usize;
    (vx, vy)
}

impl Chip {
    pub fn new(program: &str)-> Chip {
        Chip {
            regs: Registers::new(),
            mem: Memory::new(program),
            display: Display::new(),
        }
    }

    fn ret(&mut self) {
        self.regs.pc = self.mem.stack[self.regs.sp as usize];
        self.regs.sp -= 1;
    }

    fn jump(&mut self, opcode: u16) {
        self.regs.pc = opcode & 0x0fff;
    }

    fn call(&mut self, opcode: u16) {
        self.regs.sp += 1;
        self.mem.stack[self.regs.sp as usize] = self.regs.pc;
        self.jump(opcode);
    }

    fn se(&mut self, opcode: u16) {
        let (vx, k) = get_vx_k(opcode);

        if self.regs.gpr[vx] == k {
            self.regs.pc += 2;
        }
    }

    fn sne(&mut self, opcode: u16) {
        let (vx, k) = get_vx_k(opcode);

        if self.regs.gpr[vx] != k {
            self.regs.pc += 2;
        }
    }

    fn ser(&mut self, opcode: u16) {
        let (vx, vy) = get_vx_vy(opcode);

        if self.regs.gpr[vx] == self.regs.gpr[vy] {
            self.regs.pc += 2;
        }
    }

    fn ld(&mut self, opcode: u16) {
        let (vx, k) = get_vx_k(opcode);

        self.regs.gpr[vx] = k;
    }

    fn add(&mut self, opcode: u16) {
        let (vx, k) = get_vx_k(opcode);

        self.regs.gpr[vx] += k;
    }

    fn shr(&mut self, vx: usize) {
        if (self.regs.gpr[vx] & 0x01) == 0x01 {
            self.regs.gpr[0xf] = 1;
        } else {
            self.regs.gpr[0xf] = 0;
        }
        self.regs.gpr[vx] /= 2;
    }

    fn shl(&mut self, vx: usize) {
        if (self.regs.gpr[vx] & 0x80) == 0x80 {
            self.regs.gpr[0xf] = 1;
        } else {
            self.regs.gpr[0xf] = 0;
        }
        self.regs.gpr[vx] = self.regs.gpr[vx].wrapping_mul(2);
    }
    
    fn sner(&mut self, opcode: u16) {
        let (vx, vy) =get_vx_vy(opcode);
        if self.regs.gpr[vx] != self.regs.gpr[vy] {
            self.regs.pc += 2;
        }
    }

    fn addr(&mut self, opcode: u16) {
        let (vx, vy) = get_vx_vy(opcode);
        let sum: u16 = self.regs.gpr[vx] as u16 + self.regs.gpr[vy] as u16;

        if sum > 255 {
            self.regs.gpr[0xf] = 1;
        } else {
            self.regs.gpr[0xf] = 0;
        }

        self.regs.gpr[vx] = (sum & 0x00ff) as u8;
    }

    fn subr(&mut self, opcode: u16) {
        let (vx, vy) = get_vx_vy(opcode);

        if self.regs.gpr[vx] > self.regs.gpr[vy] {
            self.regs.gpr[0xf] = 1;
        } else {
            self.regs.gpr[0xf] = 0;
        }

        self.regs.gpr[vx] = self.regs.gpr[vx].wrapping_sub(self.regs.gpr[vy]);
    }

    fn subn(&mut self, opcode: u16) {
        let (vx, vy) = get_vx_vy(opcode);

        if self.regs.gpr[vy] > self.regs.gpr[vx] {
            self.regs.gpr[0xf] = 1;
        } else {
            self.regs.gpr[0xf] = 0;
        }

        self.regs.gpr[vy] = self.regs.gpr[vy].wrapping_sub(self.regs.gpr[vx]);
    }

    fn regs_operation(&mut self, opcode: u16) {
        let (vx, vy) = get_vx_vy(opcode);
        let op = opcode & 0x000f;

        match op {
            0x0 => self.regs.gpr[vx] = self.regs.gpr[vy],                      //ld
            0x1 => self.regs.gpr[vx] = self.regs.gpr[vx] | self.regs.gpr[vy],  //or
            0x2 => self.regs.gpr[vx] = self.regs.gpr[vx] & self.regs.gpr[vy],  //and
            0x3 => self.regs.gpr[vx] = self.regs.gpr[vx] ^ self.regs.gpr[vy],  //xor
            0x4 => self.addr(opcode),                                          //add
            0x5 => self.subr(opcode),                                          //sub
            0x6 => self.shr(vx),                                               //shr
            0x7 => self.subn(opcode),                                          //subn
            0xe => self.shl(vx),                                               //shl
            _ => println!("Incorrect Opcode"),
        }
    }

    fn jump_v0(&mut self, opcode: u16) {
        let op = opcode + (self.regs.gpr[0] as u16);
        self.jump(op);
    }

    fn rnd(&mut self, opcode: u16) {
        let mut rng = rand::thread_rng();
        let (vx, k) = get_vx_k(opcode);
        let r: u8 = rng.gen_range(0, 255);
        self.regs.gpr[vx] = r & k;
    }

    fn drw(&mut self, opcode: u16) {  //draws 8xn sprite. n up to 15
        //println!("drw");
        let (vx, vy) = get_vx_vy(opcode);
        let n = opcode & 0x000f;                              //number of bytes
        let (cx, cy) = (self.regs.gpr[vx], self.regs.gpr[vy]);  //coordenates
        let mut sprite: [u8; 15] = [0; 15];

        for i in 0..n {
            sprite[i as usize] = self.mem.data[(self.regs.i + i) as usize];
        }

        if self.display.draw_sprite(sprite, n, (cx, cy)) {   //if there is collision of pixels
            self.regs.gpr[0xf] = 1;
        } else{
            self.regs.gpr[0xf] = 0;
        }
    }

    fn bcd(&mut self, vx: usize) {
        //println!("bcd");
        let value = self.regs.gpr[vx];
        let h = (value / 100) % 10;
        let t = (value / 10) % 10;
        let u = value % 10;
        self.mem.data[self.regs.i as usize] = h;
        self.mem.data[(self.regs.i + 1) as usize] = t;
        self.mem.data[(self.regs.i + 2) as usize] = u;
    }

    fn input(&mut self, vx: usize) {
        //println!("input");
        let k = Term::stdout().read_key();

        match k.unwrap() {
            Key::Char('1') => self.regs.gpr[vx] = 0x1,
            Key::Char('2') => self.regs.gpr[vx] = 0x2,
            Key::Char('3') => self.regs.gpr[vx] = 0x3,
            Key::Char('4') => self.regs.gpr[vx] = 0xc,
            Key::Char('q') => self.regs.gpr[vx] = 0x4,
            Key::Char('w') => self.regs.gpr[vx] = 0x5,
            Key::Char('e') => self.regs.gpr[vx] = 0x6,
            Key::Char('r') => self.regs.gpr[vx] = 0xd,
            Key::Char('a') => self.regs.gpr[vx] = 0x7,
            Key::Char('s') => self.regs.gpr[vx] = 0x8,
            Key::Char('d') => self.regs.gpr[vx] = 0x9,
            Key::Char('f') => self.regs.gpr[vx] = 0xe,
            Key::Char('z') => self.regs.gpr[vx] = 0xa,
            Key::Char('x') => self.regs.gpr[vx] = 0x0,
            Key::Char('c') => self.regs.gpr[vx] = 0xb,
            Key::Char('v') => self.regs.gpr[vx] = 0xf,
            _ => self.input(vx),
        }
    }

    fn st(&mut self, vx: usize) {
        //println!("st");
        for i in 0..(vx + 1) {
            let position = (self.regs.i + i as u16) as usize;
            self.mem.data[position] = self.regs.gpr[i];
        }
    }

    fn load(&mut self, vx: usize) {
        //println!("load");
        for i in 0..(vx + 1) {
            let position = (self.regs.i + i as u16) as usize;
            self.regs.gpr[i] = self.mem.data[position];
        }
    }

    fn set_sprite(&mut self, vx: usize) {
        //println!("set_sprite");
        let sprite = self.regs.gpr[vx];
        self.regs.i = sprite as u16 * 5;
    }

    fn other_operation(&mut self, opcode: u16) {
        let vx = ((opcode & 0x0f00) >> 8) as usize;
        let op = opcode & 0x00ff;

        match op {
            0x07 => self.regs.gpr[vx] = self.regs.dt,          //ld Vx, dt
            0x0a => self.input(vx),                            //ld vx, k
            0x15 => self.regs.dt = self.regs.gpr[vx],          //ld dt, vx
            0x18 => self.regs.st = self.regs.gpr[vx],          //ld st, vx
            0x1e => self.regs.i += self.regs.gpr[vx] as u16,   //add i, vx
            0x29 => self.set_sprite(vx),                       //
            0x33 => self.bcd(vx),                              //ld b, vx
            0x55 => self.st(vx),                               //ld [i], vx
            0x65 => self.load(vx),                             //ld vx, [i]
            _ => println!("Incorrect Opcode"),
        }
    }

    pub fn new_cycle(&mut self) {
        let mut opcode: u16 = ((self.mem.data[self.regs.pc as usize] as u16) << 8) & 0xff00;
        self.regs.pc += 1;
        opcode = opcode | (self.mem.data[self.regs.pc as usize] as u16 & 0x00ff);
        self.regs.pc += 1;  
        
        //println!("[{}] Opcode: {}",self.regs.pc, opcode);

        let prefix = opcode >> 12;

        match prefix {
            0x0 => match opcode {
                    0x00e0 => self.display.clear(),        //cls
                    0x00ee => self.ret(),                  //ret
                    _ => (),
                },
            0x1 => self.jump(opcode),                      //jp addr
            0x2 => self.call(opcode),                      //call addr
            0x3 => self.se(opcode),                        //se Vx, byte
            0x4 => self.sne(opcode),                       //sne Vx, byte
            0x5 => self.ser(opcode),                       //se Vx, Vy
            0x6 => self.ld(opcode),                        //ld Vx, byte
            0x7 => self.add(opcode),                       //add Vx, byte
            0x8 => self.regs_operation(opcode),            //ld, or, and... Vx, Vy
            0x9 => self.sner(opcode),                      //sne Vx, Vy
            0xa => self.regs.i = opcode & 0x0fff,          //ld I, addr
            0xb => self.jump_v0(opcode),                   //jp v0, addr
            0xc => self.rnd(opcode),                       //rnd Vx, byte
            0xd => self.drw(opcode),                       //TODO: collision
            0xe => println!("TODO"),                       //TODO: cosas del teclado
            0xf => self.other_operation(opcode),           //
            _ => println!("Incorrect Opcode"),
        }
    }
}