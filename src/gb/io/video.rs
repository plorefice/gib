use super::dbg::{self, Peripheral};
use super::{IoReg, MemR, MemRW, MemSize, MemW};

#[repr(usize)]
enum Register {
    LCDC = 0x00,
    STAT = 0x01,
    SCY = 0x02,
    SCX = 0x03,
    LY = 0x04,
    // LYC = 0x05,
    // DMA = 0x06,
    BGP = 0x07,
    // OBP0 = 0x08,
    // OBP1 = 0x09,
    // WY = 0x0A,
    // WX = 0x0B,
}

#[derive(Default, Copy, Clone)]
struct Tile([u8; 16]);

impl Tile {
    fn data(&self) -> &[u8] {
        &self.0[..]
    }

    fn data_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }

    pub fn pixel(&self, x: u8, y: u8) -> u8 {
        let bl = self.0[usize::from(y) * 2];
        let bh = self.0[usize::from(y) * 2 + 1];
        (((bh >> (7 - x)) & 0x1) << 1) | ((bl >> (7 - x)) & 0x1)
    }
}

#[derive(Default, Copy, Clone)]
struct Sprite([u8; 4]);

impl Sprite {
    fn data(&self) -> &[u8] {
        &self.0[..]
    }

    fn data_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

impl<'a> MemR for &'a [Sprite] {
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
        let s = &self[usize::from(addr >> 2)];
        T::read_le(&s.data()[usize::from(addr % 2)..])
    }
}

impl<'a> MemR for &'a mut [Sprite] {
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
        (&*self as &[Sprite]).read(addr)
    }
}

impl<'a> MemW for &'a mut [Sprite] {
    fn write<T: MemSize>(&mut self, addr: u16, val: T) -> Result<(), dbg::TraceEvent> {
        let s = &mut self[usize::from(addr >> 2)];
        T::write_le(&mut s.data_mut()[usize::from(addr % 2)..], val)
    }
}

impl<'a> MemRW for &'a mut [Sprite] {}

pub struct PPU {
    tdt: [Tile; 384],  // Tile Data Table
    oam: [Sprite; 40], // Object Attribute Memory
    bgtm0: [u8; 1024], // Background Tile Map #0
    bgtm1: [u8; 1024], // Background Tile Map #1

    regs: [IoReg<u8>; 48],
    tstate: u64,
}

impl Default for PPU {
    fn default() -> PPU {
        PPU {
            tdt: [Tile::default(); 384],
            oam: [Sprite::default(); 40],
            bgtm0: [0; 1024],
            bgtm1: [0; 1024],

            regs: [IoReg::default(); 48],
            tstate: 0,
        }
    }
}

impl PPU {
    pub fn new() -> PPU {
        PPU::default()
    }

    pub fn rasterize(&self, vbuf: &mut [u8]) {
        if !self.lcdc().bit(7) {
            for b in vbuf.iter_mut() {
                *b = 0xFF;
            }
            return;
        }

        self.rasterize_bg(vbuf);
    }

    fn rasterize_bg(&self, vbuf: &mut [u8]) {
        if !self.lcdc().bit(0) {
            for b in vbuf.iter_mut() {
                *b = 0xFF;
            }
            return;
        }

        for py in 0usize..144 {
            for px in 0usize..160 {
                let y = (py + usize::from(self.scroll_y().0)) % 256;
                let x = (px + usize::from(self.scroll_x().0)) % 256;

                let pid = (py * (160 * 4)) + (px * 4);

                let t = self.bg_tile(((y >> 3) << 5) + (x >> 3));
                let px = t.pixel((x & 0x07) as u8, (y & 0x7) as u8);
                let shade = self.shade(px);

                vbuf[pid] = shade;
                vbuf[pid + 1] = shade;
                vbuf[pid + 2] = shade;
            }
        }
    }

    pub fn tick(&mut self, elapsed: u64) {
        self.tstate = (self.tstate + elapsed) % 70224;
        let v_line = self.tstate / 456;

        let mode = if v_line < 144 {
            match self.tstate % 456 {
                0..=79 => 2,   // Mode 2
                80..=279 => 3, // Mode 3
                _ => 0,        // Mode 0
            }
        } else {
            1
        };

        {
            let IoReg(ref mut stat) = self.regs[Register::STAT as usize];
            *stat = (*stat & (!0x3)) | mode;
        }
        {
            let IoReg(ref mut ly) = self.regs[Register::LY as usize];
            *ly = v_line as u8;
        }
    }

    fn lcdc(&self) -> IoReg<u8> {
        self.regs[Register::LCDC as usize]
    }

    fn bgp(&self) -> IoReg<u8> {
        self.regs[Register::BGP as usize]
    }

    fn scroll_x(&self) -> IoReg<u8> {
        self.regs[Register::SCX as usize]
    }

    fn scroll_y(&self) -> IoReg<u8> {
        self.regs[Register::SCY as usize]
    }

    fn shade(&self, color: u8) -> u8 {
        match (self.bgp().0 >> (color * 2)) & 0x3 {
            0b00 => 0xFF,
            0b01 => 0xAA,
            0b10 => 0x55,
            0b11 => 0x00,
            _ => unreachable!(),
        }
    }

    fn bg_tile(&self, id: usize) -> &Tile {
        let tile_id = if self.lcdc().bit(3) {
            self.bgtm1[id]
        } else {
            self.bgtm0[id]
        };

        if self.lcdc().bit(4) {
            &self.tdt[usize::from(tile_id)]
        } else {
            &self.tdt[(128 + i32::from(tile_id as i8)) as usize]
        }
    }
}

impl MemR for PPU {
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
        match addr {
            0x8000..=0x97FF => {
                let addr = addr - 0x8000;
                let tid = usize::from(addr >> 4);
                let bid = usize::from(addr & 0xF);
                T::read_le(&self.tdt[tid].data()[bid..])
            }
            0x9800..=0x9BFF => T::read_le(&self.bgtm0[usize::from(addr - 0x9800)..]),
            0x9C00..=0x9FFF => T::read_le(&self.bgtm1[usize::from(addr - 0x9C00)..]),
            0xFE00..=0xFE9F => (&self.oam[..]).read(addr - 0xFE00),
            0xFF40..=0xFF6F => T::read_le(&[self.regs[usize::from(addr - 0xFF40)].0]),
            _ => {
                if addr >= 0xFF00 {
                    Err(dbg::TraceEvent::IoFault(Peripheral::VPU, addr - 0xFF00))
                } else {
                    Err(dbg::TraceEvent::MemFault(addr))
                }
            }
        }
    }
}

impl MemW for PPU {
    fn write<T: MemSize>(&mut self, addr: u16, val: T) -> Result<(), dbg::TraceEvent> {
        match addr {
            0x8000..=0x97FF => {
                let addr = addr - 0x8000;
                let tid = usize::from(addr >> 4);
                let bid = usize::from(addr & 0xF);
                T::write_le(&mut self.tdt[tid].data_mut()[bid..], val)
            }
            0x9800..=0x9BFF => T::write_le(&mut self.bgtm0[usize::from(addr - 0x9800)..], val),
            0x9C00..=0x9FFF => T::write_le(&mut self.bgtm1[usize::from(addr - 0x9C00)..], val),
            0xFE00..=0xFE9F => (&mut self.oam[..]).write(addr - 0xFE00, val),
            0xFF40..=0xFF6F => {
                T::write_mut_le(&mut [&mut self.regs[usize::from(addr - 0xFF40)].0], val)
            }
            _ => {
                if addr >= 0xFF00 {
                    Err(dbg::TraceEvent::IoFault(Peripheral::VPU, addr - 0xFF00))
                } else {
                    Err(dbg::TraceEvent::MemFault(addr))
                }
            }
        }
    }
}
