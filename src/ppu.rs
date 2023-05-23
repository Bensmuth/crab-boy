// info from https://pixelbits.16-b.it/GBEDG/ppu/
// 2BPP
// In the Gameboy’s 2BPP format, 2 bytes make up a row of 8 pixels. Each bit of
// the first byte is combined with the bit at the same position of the second
// byte to calculate the color number
//
// Layers
// Has 3 seperate layers:
//
// * The background
//  - 32x32 tile grid in which tiles can be placed
//  - Lcd viewport is only 20x18
//  - The position of the viewport is modified by scx and scy
//  - wraps if gone over the edge
//
// * The window
//  - similar to the background but drawn over it ("overlay")
//  - position is determened using wx and wy registers
//  * WY - y pos which the top border of the window should be placed, 0 being the top
//  * WX - horizontal position, calculated using WX-7 , a valu eof 6 for example puts the leftmost column of pixels off screen and starts renderin ght ewindow at the second column of pixels
//
// * Sprites
//  - 8x8 (or sometimtes 8x16)
//  - stored in OAM section of memory (can store up to 40 sprites)
//
//
//
// * Tile data
//  - stored from 0x8000 to 0x97ff
//  - 2 addressing methods
//  * 8000 method
//   - address via 0x8000+(TILE_NUMBER * 16)
//   - tile number is unsigned 8-bit integer
//  * 8800 method
//   - address via 0x9000+(SIGNED_TILE_NUMBER*16)
//   - tile number is a signed 9 bit integer
//  - addressing method depends on bit 4 of the lcdc register
//
//  * Background maps
//   - dictates what should be displayed in the background / window grids
//    - map 1: 0x9800-0x9BFF
//    - map 2: 0x9C00-0x9FFF
//   - consists of 32x32 bytes representing tile numbers irganized row by row
//
//  * OAM memory
//   - object attribute memory
//   - 0xFE00-0xFE9F
//   - each sprite takes up 4 bytes in this bit of memory
//    - byte 0 - y position, (byte - 16)
//    - byte 1 - x position. (byte - 8)
//    - byte 2 - tile number. uses 8000 addressing so always unsigned
//    - byte 3 - sprite flags (TODO look into this)
//
//  * Scanlines
//   - just gonna skip this cause im lazy - LY is the current scanline
//
//  PPU modes
//  * OAM Scan (mode 2)
//  - entered at the start of every scanline (except for
//    V-Blank) before pixels are actually drawn to the screen. During this mode
//    the PPU searches OAM memory for sprites that should be rendered on the
//    current scanline and stores them in a buffer.
//  - A sprite is only added to the buffer if all of the following conditions apply:    
//     1. Sprite X-Position must be greater than 0
//     2. LY + 16 must be greater than or equal to Sprite Y-Position
//     3. LY + 16 must be less than Sprite Y-Position + Sprite Height (8 in Normal Mode, 16 in Tall-Sprite-Mode)
//     4. The amount of sprites already stored in the OAM Buffer must be less than 10
//  * Drawing (Mode 3)
//   - PPU transfers pixels to the LCD
//
//  * H-Blank (Mode 0)
//   - time after drawing a scanline - ppu pauses during this
//  * V-blank
//   - time after adrawing a frame, so after the 144 scanlines 10 more scanlines exist

pub struct Ppu {
    
}

impl Ppu {
    fn new(){
        
    }
    fn frame()
}
