use super::dbg;
use super::{InterruptSource, IrqSource};
use super::{IoReg, MemR, MemRW, MemSize, MemW};

/// A Tile is the bit representation of an 8x8 sprite or BG tile,
/// with a color depth of 4 colors/gray shades.
///
/// Each Tile occupies 16 bytes, where each 2 bytes represent a line:
///    Byte 0-1  First Line (Upper 8 pixels)
///    Byte 2-3  Next Line
///    etc.
/// For each line, the first byte defines the least significant bits of the color numbers
/// for each pixel, and the second byte defines the upper bits of the color numbers.
/// In either case, Bit 7 is the leftmost pixel, and Bit 0 the rightmost.
#[derive(Default, Copy, Clone)]
struct Tile([u8; 16]);

impl Tile {
    fn data(&self) -> &[u8] {
        &self.0[..]
    }

    fn data_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }

    /// Returns the shade associated with pixel (x,y) in the Tile.
    pub fn pixel(&self, x: u8, y: u8) -> u8 {
        let bl = self.0[usize::from(y) * 2];
        let bh = self.0[usize::from(y) * 2 + 1];
        (((bh >> (7 - x)) & 0x1) << 1) | ((bl >> (7 - x)) & 0x1)
    }
}

/// A Sprite is an entry in the Sprite Attribute Table (or OAM - Object Attribute Memory).
///
/// Each Sprite consists of 4 bytes representing the sprite's position, associated tile and attributes.
#[derive(Default, Copy, Clone)]
struct Sprite {
    y: u8,
    x: u8,
    tid: u8,
    attributes: SpriteAttributes,
}

bitflags! {
    struct SpriteAttributes: u8 {
        const BG_PRIO = 0b_1000_0000;
        const FLIP_X  = 0b_0100_0000;
        const FLIP_Y  = 0b_0010_0000;
        const PAL_NUM = 0b_0001_0000;

        const DEFAULT = 0b_0000_0000;
    }
}

// On DMG the sprite flags have unused bits, but they are still writable and readable normally.
mem_rw!(SpriteAttributes, 0x00);

impl Default for SpriteAttributes {
    fn default() -> SpriteAttributes {
        SpriteAttributes::DEFAULT
    }
}

impl<'a> MemR for &'a [Sprite] {
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
        let s = &self[usize::from(addr >> 2)];

        match addr % 4 {
            0 => T::read_le(&[s.y]),
            1 => T::read_le(&[s.x]),
            2 => T::read_le(&[s.tid]),
            3 => (&s.attributes).read(0),
            _ => unreachable!(),
        }
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

        match addr % 4 {
            0 => T::write_mut_le(&mut [&mut s.y], val),
            1 => T::write_mut_le(&mut [&mut s.x], val),
            2 => T::write_mut_le(&mut [&mut s.tid], val),
            3 => (&mut s.attributes).write(0, val),
            _ => unreachable!(),
        }
    }
}

impl<'a> MemRW for &'a mut [Sprite] {}

bitflags! {
    /// FF40 - LCDC - LCD Control (R/W)
    struct LCDC: u8 {
        const DISP_EN         = 0b_1000_0000; /// Bit 7 - LCD Display Enable             (0=Off, 1=On)
        const WIN_DISP_SEL    = 0b_0100_0000; /// Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
        const WIN_DISP_EN     = 0b_0010_0000; /// Bit 5 - Window Display Enable          (0=Off, 1=On)
        const BG_WIN_DATA_SEL = 0b_0001_0000; /// Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
        const BG_DISP_SEL     = 0b_0000_1000; /// Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
        const OBJ_SIZE        = 0b_0000_0100; /// Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
        const OBJ_DISP_EN     = 0b_0000_0010; /// Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
        const BG_DISP         = 0b_0000_0001; /// Bit 0 - BG Display (for CGB see below) (0=Off, 1=On)

        const DEFAULT = 0b_1001_0001;
    }
}

mem_rw!(LCDC, 0x00);

bitflags! {
    /// FF41 - STAT - LCDC Status (R/W)
    struct STAT: u8 {
        const LYC_INTR = 0b_0100_0000; /// Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
        const OAM_INTR = 0b_0010_0000; /// Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
        const VBK_INTR = 0b_0001_0000; /// Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
        const HBK_INTR = 0b_0000_1000; /// Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
        const LYC_FLAG = 0b_0000_0100; /// Bit 2 - Coincidence Flag  (0:LYC<>LY, 1:LYC=LY) (Read Only)
        const MOD_FLAG = 0b_0000_0011; /// Bit 1-0 - Mode Flag       (Mode 0-3, see below) (Read Only)

        const MOD_0    = 0b_0000_0000;
        const MOD_1    = 0b_0000_0001;
        const MOD_2    = 0b_0000_0010;
        const MOD_3    = 0b_0000_0011;

        const DEFAULT = 0b_0000_0000;
    }
}

mem_rw!(STAT, 0x80);

bitflags! {
    /// Used to keep track of which STAT IRQs are currently active.
    struct STATIRQ: u8 {
        const LYC = 0b_0100_0000;
        const OAM = 0b_0010_0000;
        const VBK = 0b_0001_0000;
        const HBK = 0b_0000_1000;

        const DEFAULT = 0b_0000_0000;
    }
}

pub struct PPU {
    tdt: [Tile; 384],  // Tile Data Table
    oam: [Sprite; 40], // Object Attribute Memory
    bgtm0: [u8; 1024], // Background Tile Map #0
    bgtm1: [u8; 1024], // Background Tile Map #1

    // Ctrl/status IO registes
    lcdc_reg: LCDC,
    stat_reg: STAT,
    stat_irq: STATIRQ,

    // Position/scrolling registers
    scx_reg: IoReg<u8>,
    scy_reg: IoReg<u8>,
    lyc_reg: IoReg<u8>,
    ly_reg: IoReg<u8>,
    wy_reg: IoReg<u8>,
    wx_reg: IoReg<u8>,

    // Monochorome palette registers
    obp0_reg: IoReg<u8>,
    obp1_reg: IoReg<u8>,
    bgp_reg: IoReg<u8>,

    // DMA register & counter
    dma_reg: IoReg<u8>,
    dma_xfer_cycle: u64,

    // Timings
    tstate: u64,

    // IRQ handling
    vblank_irq_pending: bool,
}

impl Default for PPU {
    fn default() -> PPU {
        PPU {
            tdt: [Tile::default(); 384],
            oam: [Sprite::default(); 40],
            bgtm0: [0; 1024],
            bgtm1: [0; 1024],

            lcdc_reg: LCDC::DEFAULT,
            stat_reg: STAT::DEFAULT,
            stat_irq: STATIRQ::DEFAULT,

            scx_reg: IoReg(0x00),
            scy_reg: IoReg(0x00),
            lyc_reg: IoReg(0x00),
            ly_reg: IoReg(0x99),
            wy_reg: IoReg(0x00),
            wx_reg: IoReg(0x00),

            bgp_reg: IoReg(0xFC),
            obp0_reg: IoReg(0xFF),
            obp1_reg: IoReg(0xFF),

            dma_reg: IoReg(0x00),
            dma_xfer_cycle: 0,

            tstate: 70164,

            vblank_irq_pending: true,
        }
    }
}

impl PPU {
    pub fn new() -> PPU {
        PPU::default()
    }

    /// Advances the LCD controller state machine by a single M-cycle.
    pub fn tick(&mut self) {
        // No tick should be performed if the LCD display is disabled
        if !self.lcdc_reg.contains(LCDC::DISP_EN) {
            return;
        }

        self.tstate = (self.tstate + 4) % 70224;
        let tstate = self.tstate % 456;
        let v_line = self.tstate / 456;

        self.ly_reg.0 = v_line as u8;

        // V-Blank IRQ happens at the beginning of the 144th line
        if v_line == 144 && tstate == 0 {
            self.vblank_irq_pending = true;
        }

        // This should be called last, after every other counter has been updated!
        self.tick_stat(tstate, v_line);
    }

    /// Returns a pair of source and destination addresses for DMA transfer
    /// if one is currently in progress, otherwise `None`.
    pub fn advance_dma_xfer(&mut self) -> Option<(u16, u16)> {
        if self.dma_xfer_cycle > 0 {
            let n = (160 - self.dma_xfer_cycle) as u16;

            let src = (u16::from(self.dma_reg.0) << 8) + n;
            let dst = 0xFE00 + n;

            self.dma_xfer_cycle -= 1;

            Some((src, dst))
        } else {
            None
        }
    }

    /// Rasterizes the current contents of the Video RAM to the provided video buffer.
    ///
    /// NOTE: the buffer is assumed to be in U8U8U8U8 RGBA format.
    pub fn rasterize(&self, vbuf: &mut [u8]) {
        // When the LCD display is disabled, show a white screen
        if !self.lcdc_reg.contains(LCDC::DISP_EN) {
            for b in vbuf.iter_mut() {
                *b = 0xFF;
            }
            return;
        }

        // Draw BG, Window and sprites
        self.rasterize_bg(vbuf);
        self.rasterize_sprites(vbuf);
    }

    /// Rasterizes the current background map to the video buffer.
    fn rasterize_bg(&self, vbuf: &mut [u8]) {
        // When BG displaying is disabled, show a white background
        if !self.lcdc_reg.contains(LCDC::BG_DISP) {
            for b in vbuf.iter_mut() {
                *b = 0xFF;
            }
            return;
        }

        // TODO: implement Window support
        if self.lcdc_reg.contains(LCDC::WIN_DISP_EN) {
            unimplemented!();
        }

        // Iterate over each pixel in the screen
        for py in 0usize..144 {
            for px in 0usize..160 {
                // Wrap to the top-left in case the scroll registers cause any overflows
                let y = (py + usize::from(self.scy_reg.0)) % 256;
                let x = (px + usize::from(self.scx_reg.0)) % 256;

                // Retrieve the tile corresponding to pixel (x,y) by partitioning the
                // screen area in 8x8 tiles
                let tile = self.get_bg_tile(((y >> 3) << 5) + (x >> 3));

                // Obtain the color of the tile's pixel corresponding to (x,y)
                let pixel = tile.pixel((x & 0x07) as u8, (y & 0x7) as u8);
                let shade = self.get_shade(self.bgp_reg.0, pixel);

                // Compute the index in the video buffer
                let pid = (py * (160 * 4)) + (px * 4);

                vbuf[pid] = shade;
                vbuf[pid + 1] = shade;
                vbuf[pid + 2] = shade;
            }
        }
    }

    /// Rasterizes any visible sprite to the video buffer.
    fn rasterize_sprites(&self, vbuf: &mut [u8]) {
        // Do nothing if sprite displaying is disabled
        if !self.lcdc_reg.contains(LCDC::OBJ_DISP_EN) {
            return;
        }

        // TODO implement 8x16 sprites
        if self.lcdc_reg.contains(LCDC::OBJ_SIZE) {
            unimplemented!();
        }

        let scy = i16::from(self.scy_reg.0);
        let scx = i16::from(self.scx_reg.0);

        for sprite in self.oam.iter() {
            let y = i16::from(sprite.y) - 16 - scy;
            let x = i16::from(sprite.x) - 8 - scx;
            let attr = sprite.attributes;

            let tile = self.get_sprite_tile(sprite.tid.into());

            // The palette used in rasterizing the srpite depends on its attributes
            let palette = if attr.contains(SpriteAttributes::PAL_NUM) {
                self.obp1_reg.0
            } else {
                self.obp0_reg.0
            };

            // Flip sprite horizontally
            let off_x = if attr.contains(SpriteAttributes::FLIP_X) {
                7
            } else {
                0
            };

            // Flip sprite vertically
            let off_y = if attr.contains(SpriteAttributes::FLIP_Y) {
                7
            } else {
                0
            };

            // TODO put the sprite behind BG colors 1-3
            let _behind_bg = attr.contains(SpriteAttributes::BG_PRIO);

            // Clip to currently visible area
            for py in y.max(0)..(y + 8).min(144) {
                for px in x.max(0)..(x + 8).min(160) {
                    let x = (off_x - (px - x) as i16).abs() as u8;
                    let y = (off_y - (py - y) as i16).abs() as u8;

                    let pixel = tile.pixel(x, y);
                    let shade = self.get_shade(palette, pixel);

                    let pid = (py as usize) * 160 * 4 + (px as usize) * 4;

                    if pixel != 0 {
                        vbuf[pid] = shade;
                        vbuf[pid + 1] = shade;
                        vbuf[pid + 2] = shade;
                    }
                }
            }
        }
    }

    /// Update the STAT register and set any relevant interrupts.
    fn tick_stat(&mut self, tstate: u64, v_line: u64) {
        // Compute current LCD mode
        let mode = if v_line < 144 {
            match tstate {
                0..=79 => STAT::MOD_2,
                80..=253 => STAT::MOD_3,
                _ => STAT::MOD_0,
            }
        } else {
            STAT::MOD_1
        };

        let lyc_coinc = self.ly_reg == self.lyc_reg;

        // Set STAT interrupt flags depending on the enable bits in STAT
        if self.stat_reg.contains(STAT::LYC_INTR) && lyc_coinc && tstate == 0 {
            self.stat_irq |= STATIRQ::LYC;
        }
        if self.stat_reg.contains(STAT::OAM_INTR) && mode == STAT::MOD_2 && tstate == 0 {
            self.stat_irq |= STATIRQ::OAM;
        }
        if self.stat_reg.contains(STAT::VBK_INTR) && v_line == 144 && tstate == 0 {
            self.stat_irq |= STATIRQ::VBK;
        }
        if self.stat_reg.contains(STAT::HBK_INTR) && mode == STAT::MOD_0 && tstate == 256 {
            self.stat_irq |= STATIRQ::HBK;
        }

        // Update coincidence flag
        if lyc_coinc {
            self.stat_reg |= STAT::LYC_FLAG;
        } else {
            self.stat_reg &= !STAT::LYC_FLAG;
        }

        // Update mode flag
        self.stat_reg = (self.stat_reg & !STAT::MOD_FLAG) | mode;
    }

    /// Initiates a new DMA transfer from RAM or ROM to OAM.
    ///
    /// The transfer lasts 160 cycles, during which the CPU can only access HRAM.
    fn prepare_dma_xfer<T: MemSize>(&mut self, val: T) -> Result<(), dbg::TraceEvent> {
        if self.dma_xfer_cycle == 0 {
            self.dma_reg.0 = val.low();
            self.dma_xfer_cycle = 160;
        }
        Ok(())
    }

    /// Returns the actual gray shade associated with a pixel value in a palette.
    fn get_shade(&self, palette: u8, pixel: u8) -> u8 {
        match (palette >> (pixel * 2)) & 0x3 {
            0b00 => 0xFF, // White
            0b01 => 0xAA, // Light gray
            0b10 => 0x55, // Dark gray
            0b11 => 0x00, // Black
            _ => unreachable!(),
        }
    }

    /// Returns the BG tile corresponding to the given ID.
    ///
    /// The resulting Tile depends on the selected BG Tile Map and addressing mode
    /// in LCDC register.
    fn get_bg_tile(&self, id: usize) -> &Tile {
        let tile_id = if self.lcdc_reg.contains(LCDC::BG_DISP_SEL) {
            self.bgtm1[id]
        } else {
            self.bgtm0[id]
        };

        if self.lcdc_reg.contains(LCDC::BG_WIN_DATA_SEL) {
            &self.tdt[usize::from(tile_id)]
        } else {
            &self.tdt[(256 + i32::from(tile_id as i8)) as usize]
        }
    }

    /// Returns the sprite tile corresponding to the given ID.
    fn get_sprite_tile(&self, id: usize) -> &Tile {
        // TODO support loading 8x16 sprites
        &self.tdt[id]
    }
}

impl InterruptSource for PPU {
    fn get_and_clear_irq(&mut self) -> Option<IrqSource> {
        if self.vblank_irq_pending {
            self.vblank_irq_pending = false;
            Some(IrqSource::VBlank)
        } else if !self.stat_irq.is_empty() {
            // TODO: resetting everything might not be the correct behavior.
            self.stat_irq = STATIRQ::DEFAULT;
            Some(IrqSource::LcdStat)
        } else {
            None
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

            0xFF40 => (&self.lcdc_reg).read(addr),
            0xFF41 => (&self.stat_reg).read(addr),
            0xFF42 => T::read_le(&[self.scy_reg.0]),
            0xFF43 => T::read_le(&[self.scx_reg.0]),
            0xFF44 => T::read_le(&[self.ly_reg.0]),
            0xFF45 => T::read_le(&[self.lyc_reg.0]),
            0xFF46 => T::read_le(&[0xFF]),
            0xFF47 => T::read_le(&[self.bgp_reg.0]),
            0xFF48 => T::read_le(&[self.obp0_reg.0]),
            0xFF49 => T::read_le(&[self.obp1_reg.0]),
            0xFF4A => T::read_le(&[self.wy_reg.0]),
            0xFF4B => T::read_le(&[self.wx_reg.0]),

            _ => unreachable!(),
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

            0xFF40 => (&mut self.lcdc_reg).write(0, val),
            0xFF41 => (&mut self.stat_reg).write(0, val),
            0xFF42 => T::write_mut_le(&mut [&mut self.scy_reg.0], val),
            0xFF43 => T::write_mut_le(&mut [&mut self.scx_reg.0], val),
            0xFF44 => Ok(()),
            0xFF45 => T::write_mut_le(&mut [&mut self.lyc_reg.0], val),
            0xFF46 => self.prepare_dma_xfer(val),
            0xFF47 => T::write_mut_le(&mut [&mut self.bgp_reg.0], val),
            0xFF48 => T::write_mut_le(&mut [&mut self.obp0_reg.0], val),
            0xFF49 => T::write_mut_le(&mut [&mut self.obp1_reg.0], val),
            0xFF4A => T::write_mut_le(&mut [&mut self.wy_reg.0], val),
            0xFF4B => T::write_mut_le(&mut [&mut self.wx_reg.0], val),

            _ => unreachable!(),
        }
    }
}
