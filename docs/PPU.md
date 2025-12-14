# PPU Memory Map
| Address range |  Size |       Description      |    Mapped by     |
|:-------------:|:-----:|:----------------------:|:----------------:|
| $0000-$0FFF   | $1000 | Pattern table 0        | Cartridge        |
| $1000-$1FFF   | $1000 | Pattern table 1        | Cartridge        |
| $2000-$23BF   | $03c0 | Nametable 0            | Cartridge        |
| $23C0-$23FF   | $0040 | Attribute table 0      | Cartridge        |
| $2400-$27BF   | $03c0 | Nametable 1            | Cartridge        |
| $27C0-$27FF   | $0040 | Attribute table 1      | Cartridge        |
| $2800-$2BBF   | $03c0 | Nametable 2            | Cartridge        |
| $2BC0-$2BFF   | $0040 | Attribute table 2      | Cartridge        |
| $2C00-$2FBF   | $03c0 | Nametable 3            | Cartridge        |
| $2FC0-$2FFF   | $0040 | Attribute table 3      | Cartridge        |
| $3000-$3EFF   | $0F00 | Unused                 | Cartridge        |
| $3F00-$3F0F   | $0010 | Bg palette RAM indexes | Internal to PPU  |
| $3F10-$3F1F   | $0010 | Fg palette RAM indexes | Internal to PPU  |
| $3F20-$3FFF   | $00E0 | Mirrors of $3F00-$3F1F | Internal to PPU  |

## Pattern table 

The pattern table is an area of memory connected to the PPU that defines the shapes of tiles that make up backgrounds and sprites.
Each tile in the pattern table is 16 bytes, made of two planes. Each bit in the first plane controls bit 0 of a pixel's color index; the corresponding bit in the second plane controls bit 1.

## Nametable

A nametable is a 1024 byte area of memory used by the PPU to lay out backgrounds. Each byte in the nametable controls one 8x8 pixel character cell, and each nametable has 30 rows of 32 tiles each, for 960 ($3C0) bytes; the 64 ($40) remaining bytes are used by each nametable's attribute table. With each tile being 8x8 pixels, this makes a total of 256x240 pixels in one map, the same size as one full screen. 

Each tile is basically an index to the pattern table

### Attribute table

An attribute table is a 64-byte array at the end of each nametable that controls which palette is assigned to each part of the background.

Each attribute table, starting at $23C0, $27C0, $2BC0, or $2FC0, is arranged as an 8x8 byte array: 
```
       2xx0    2xx1    2xx2    2xx3    2xx4    2xx5    2xx6    2xx7
     ,-------+-------+-------+-------+-------+-------+-------+-------.
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
2xC0:| - + - | - + - | - + - | - + - | - + - | - + - | - + - | - + - |
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
     +-------+-------+-------+-------+-------+-------+-------+-------+
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
2xC8:| - + - | - + - | - + - | - + - | - + - | - + - | - + - | - + - |
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
     +-------+-------+-------+-------+-------+-------+-------+-------+
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
2xD0:| - + - | - + - | - + - | - + - | - + - | - + - | - + - | - + - |
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
     +-------+-------+-------+-------+-------+-------+-------+-------+
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
2xD8:| - + - | - + - | - + - | - + - | - + - | - + - | - + - | - + - |
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
     +-------+-------+-------+-------+-------+-------+-------+-------+
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
2xE0:| - + - | - + - | - + - | - + - | - + - | - + - | - + - | - + - |
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
     +-------+-------+-------+-------+-------+-------+-------+-------+
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
2xE8:| - + - | - + - | - + - | - + - | - + - | - + - | - + - | - + - |
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
     +-------+-------+-------+-------+-------+-------+-------+-------+
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
2xF0:| - + - | - + - | - + - | - + - | - + - | - + - | - + - | - + - |
     |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
     +-------+-------+-------+-------+-------+-------+-------+-------+
2xF8:|   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
     `-------+-------+-------+-------+-------+-------+-------+-------'
```

Each byte controls the palette of a 32×32 pixel or 4×4 tile part of the nametable and is divided into four 2-bit areas. Meaning:

```
7654 3210
|||| ||++- Color bits 3-2 for top left quadrant of this byte
|||| ++--- Color bits 3-2 for top right quadrant of this byte
||++------ Color bits 3-2 for bottom left quadrant of this byte
++-------- Color bits 3-2 for bottom right quadrant of this byte
```

## Palette RAM indexes

Save 32 bytes of palette RAM (PPU $3F00–$3F1F)
Each byte is one color (even though only 6 bits are used), a single pallette is 4 bytes (while the first one is transparent).

Indexing into the pallette table is done the following way:
```
4bit0
-----
SAAPP
|||||
|||++- Pixel value from tile pattern data
|++--- Palette number from attributes
+----- Background/Sprite select
```

# PPU Rendering

