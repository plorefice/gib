use super::bus::{MemR, MemRW, MemSize, MemW};

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

impl<'a, T: MemSize> MemR<T> for &'a [Sprite] {
    fn read(&self, addr: u16) -> T {
        let s = &self[usize::from(addr >> 2)];
        T::read_le(&s.data()[usize::from(addr % 2)..])
    }
}

impl<'a, T: MemSize> MemR<T> for &'a mut [Sprite] {
    fn read(&self, addr: u16) -> T {
        (self as &MemR<T>).read(addr)
    }
}

impl<'a, T: MemSize> MemW<T> for &'a mut [Sprite] {
    fn write(&mut self, addr: u16, val: T) {
        let s = &mut self[usize::from(addr >> 2)];
        T::write_le(&mut s.data_mut()[usize::from(addr % 2)..], val);
    }
}

impl<'a, T: MemSize> MemRW<T> for &'a mut [Sprite] {}

pub struct PPU {
    tdt: [Tile; 384],  // Tile Data Table
    oam: [Sprite; 40], // Object Attribute Memory
    bgtm0: [u8; 1024], // Background Tile Map #0
    bgtm1: [u8; 1024], // Background Tile Map #1

    regs: [u8; 48],
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            tdt: [Tile::default(); 384],
            oam: [Sprite::default(); 40],
            bgtm0: [0; 1024],
            bgtm1: [0; 1024],
            regs: [0; 48],
        }
    }

    pub fn rasterize(&self, vbuf: &mut [u8]) {
        for y in 0usize..256 {
            for x in 0usize..256 {
                let pid = (y * 1024) + (x * 4);

                let t = self.tile_at(((y >> 3) << 5) + (x >> 3));
                let px = t.pixel((x & 0x07) as u8, (y & 0x7) as u8);
                let shade = self.shade(px);

                vbuf[pid] = shade;
                vbuf[pid + 1] = shade;
                vbuf[pid + 2] = shade;
            }
        }
    }

    fn io_read<T: MemSize>(&self, idx: u16) -> T {
        T::read_le(&self.regs[usize::from(idx)..])
    }

    fn io_write<T: MemSize>(&mut self, idx: u16, v: T) {
        T::write_le(&mut self.regs[usize::from(idx)..], v);
    }

    fn bgp(&self) -> u8 {
        self.regs[0x7]
    }

    fn shade(&self, color: u8) -> u8 {
        match (self.bgp() >> (color * 2)) & 0x3 {
            0 => 0xFF,
            1 => 0xAA,
            2 => 0x55,
            3 => 0x00,
            _ => unreachable!(),
        }
    }

    fn tile_at(&self, bgid: usize) -> &Tile {
        let tid = self.bgtm0[bgid];
        &self.tdt[usize::from(tid)]
    }
}

impl<T: MemSize> MemR<T> for PPU {
    fn read(&self, addr: u16) -> T {
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
            0xFF40..=0xFF6F => self.io_read(addr - 0xFF40),
            _ => unreachable!(),
        }
    }
}

impl<T: MemSize> MemW<T> for PPU {
    fn write(&mut self, addr: u16, val: T) {
        match addr {
            0x8000..=0x97FF => {
                let addr = addr - 0x8000;
                let tid = usize::from(addr >> 4);
                let bid = usize::from(addr & 0xF);
                T::write_le(&mut self.tdt[tid].data_mut()[bid..], val);
            }
            0x9800..=0x9BFF => T::write_le(&mut self.bgtm0[usize::from(addr - 0x9800)..], val),
            0x9C00..=0x9FFF => T::write_le(&mut self.bgtm1[usize::from(addr - 0x9C00)..], val),
            0xFE00..=0xFE9F => (&mut self.oam[..]).write(addr - 0xFE00, val),
            0xFF40..=0xFF6F => self.io_write(addr - 0xFF40, val),
            _ => unreachable!(),
        }
    }
}
