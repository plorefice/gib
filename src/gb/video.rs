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

pub struct VPU {
    tdt: [Tile; 384],  // Tile Data Table
    oam: [Sprite; 40], // Object Attribute Memory
    bgtm0: [u8; 1024], // Background Tile Map #0
    bgtm1: [u8; 1024], // Background Tile Map #1
}

impl VPU {
    pub fn new() -> VPU {
        VPU {
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
}

impl<T: MemSize> MemR<T> for VPU {
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

impl<T: MemSize> MemW<T> for VPU {
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
