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
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            tdt: [Tile::default(); 384],
            oam: [Sprite::default(); 40],
            bgtm0: [0; 1024],
            bgtm1: [0; 1024],
        }
    }

    pub fn oam<'a, T: MemSize>(&'a self) -> impl MemR<T> + 'a {
        &self.oam[..]
    }

    pub fn oam_mut<'a, T: MemSize>(&'a mut self) -> impl MemRW<T> + 'a {
        &mut self.oam[..]
    }

    pub fn rasterize(&self, vbuf: &mut [u8]) {
        for y in 0usize..256 {
            for x in 0usize..256 {
                let pid = (y * 1024) + (x * 4);

                let t = self.tile_at(((y >> 3) << 5) + (x >> 3));
                let px = t.pixel((x & 0x07) as u8, (y & 0x7) as u8);

                vbuf[pid] = if px == 0 { 0x00 } else { 0xFF };
                vbuf[pid + 1] = if px == 0 { 0x00 } else { 0xFF };
                vbuf[pid + 2] = if px == 0 { 0x00 } else { 0xFF };
            }
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
            0x0000..=0x17FF => {
                let tid = usize::from(addr >> 4);
                let bid = usize::from(addr & 0xF);
                T::read_le(&self.tdt[tid].data()[bid..])
            }
            0x1800..=0x1BFF => T::read_le(&self.bgtm0[usize::from(addr - 0x1800)..]),
            0x1C00..=0x1FFF => T::read_le(&self.bgtm1[usize::from(addr - 0x1C00)..]),
            _ => unreachable!(),
        }
    }
}

impl<T: MemSize> MemW<T> for PPU {
    fn write(&mut self, addr: u16, val: T) {
        match addr {
            0x0000..=0x17FF => {
                let tid = usize::from(addr >> 4);
                let bid = usize::from(addr & 0xF);
                T::write_le(&mut self.tdt[tid].data_mut()[bid..], val);
            }
            0x1800..=0x1BFF => T::write_le(&mut self.bgtm0[usize::from(addr - 0x1800)..], val),
            0x1C00..=0x1FFF => T::write_le(&mut self.bgtm1[usize::from(addr - 0x1C00)..], val),
            _ => unreachable!(),
        }
    }
}
