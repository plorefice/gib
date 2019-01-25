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

impl<T: MemSize> MemR<T> for [Sprite; 40] {
    fn read(&self, addr: u16) -> T {
        let s = &self[usize::from(addr >> 2)];
        T::read_le(&s.data()[usize::from(addr % 2)..])
    }
}

impl<T: MemSize> MemW<T> for [Sprite; 40] {
    fn write(&mut self, addr: u16, val: T) {
        let s = &mut self[usize::from(addr >> 2)];
        T::write_le(&mut s.data_mut()[usize::from(addr % 2)..], val);
    }
}

impl<T: MemSize> MemRW<T> for [Sprite; 40] {}

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

    pub fn oam<T: MemSize>(&self) -> &MemR<T> {
        &self.oam
    }

    pub fn oam_mut<T: MemSize>(&mut self) -> &mut MemRW<T> {
        &mut self.oam
    }

    pub fn rasterize(&self, vbuf: &mut [u8]) {
        for y in 0usize..256 {
            for x in 0usize..256 {
                let pid = (y * 1024) + (x * 4);

                let t = self.tile_at((((y >> 3) << 5) + (x >> 3)) as u8);
                let px = t.pixel((x % 8) as u8, (y % 8) as u8);

                vbuf[pid] = if px == 0 { 0x00 } else { 0xFF };
                vbuf[pid + 1] = if px == 0 { 0x00 } else { 0xFF };
                vbuf[pid + 2] = if px == 0 { 0x00 } else { 0xFF };
                vbuf[pid + 3] = 0;
            }
        }
    }

    fn tile_at(&self, idx: u8) -> &Tile {
        &self.tdt[(128 + isize::from(idx as i8)) as usize]
    }
}

impl<T: MemSize> MemR<T> for PPU {
    fn read(&self, addr: u16) -> T {
        match addr {
            0x0000..=0x17FF => {
                let tid = usize::from(addr / 16);
                let bid = usize::from(addr % 16);
                T::read_le(&self.tdt[tid].data()[bid..])
            }
            0x1800..=0x1BFF => panic!("Background Tile Map #0 not implemented"),
            0x1C00..=0x1FFF => panic!("Background Tile Map #0 not implemented"),
            _ => unreachable!(),
        }
    }
}

impl<T: MemSize> MemW<T> for PPU {
    fn write(&mut self, addr: u16, val: T) {
        match addr {
            0x0000..=0x17FF => {
                let tid = usize::from(addr / 16);
                let bid = usize::from(addr % 16);
                T::write_le(&mut self.tdt[tid].data_mut()[bid..], val);
            }
            0x1800..=0x1BFF => T::write_le(&mut self.bgtm0[usize::from(addr - 0x1800)..], val),
            0x1C00..=0x1FFF => T::write_le(&mut self.bgtm1[usize::from(addr - 0x1C00)..], val),
            _ => unreachable!(),
        }
    }
}
