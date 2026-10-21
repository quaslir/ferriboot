use crate::ParseError;
use {crate::write_u16, crate::write_u32};
const ORIG_VIDEO_IS_VGA_OFFSET: u8 = 0x0F;
const LFB_WIDTH_OFFSET: u16 = 0x12;
const LFB_HEIGHT_OFFSET: u16 = 0x14;
const LFB_DEPTH_OFFSET: u16 = 0x16;
const LFB_BASE_OFFSET: u32 = 0x18;
const LFB_LINELENGTH_OFFSET: u16 = 0x24;
// Order: red, green, blue, reserved
const CHANNEL_SIZE_OFFSETS: [usize; 4] = [0x26, 0x28, 0x2A, 0x2C];
const CHANNEL_POS_OFFSETS: [usize; 4] = [0x27, 0x29, 0x2B, 0x2D];

const CAPABILITIES_OFFSET: u32 = 0x36;
const EXT_LFB_BASE_OFFSET: u32 = 0x3A;
const LFB_SIZE_OFFSET: u32 = 0x1C;

const VIDEO_TYPE_EFI: u8 = 0x70;
const VIDEO_CAPABILITY_64BIT_BASE: u32 = 1 << 1;
const LFB_DEPTH_32: u16 = 32;
const CHANNEL_SIZE: u8 = 8;

pub fn fill_screen_info(
    params: &mut [u8],
    base: u64,
    size: u64,
    w: u16,
    h: u16,
    stride: u16,
    is_bgr: bool,
) -> Result<(), ParseError> {
    params[ORIG_VIDEO_IS_VGA_OFFSET as usize] = VIDEO_TYPE_EFI;
    write_u16(params, LFB_WIDTH_OFFSET as usize, w);
    write_u16(params, LFB_HEIGHT_OFFSET as usize, h);
    write_u16(params, LFB_DEPTH_OFFSET as usize, LFB_DEPTH_32);
    let lfb_line_l = u16::checked_mul(stride, 4).ok_or(ParseError::FramebufferError)?;
    write_u16(params, LFB_LINELENGTH_OFFSET as usize, lfb_line_l);
    write_u32(params, LFB_BASE_OFFSET as usize, base as u32);
    let size_u32 = u32::try_from(size).map_err(|_| ParseError::FramebufferError)?;
    write_u32(params, LFB_SIZE_OFFSET as usize, size_u32);

    for i in CHANNEL_SIZE_OFFSETS {
        // First 4 values are the same for RGB and BGR
        params[i] = CHANNEL_SIZE;
    }
    let rb_values: (u8, u8);
    if is_bgr {
        rb_values = (16, 0);
    } else {
        rb_values = (0, 16);
    }

    params[CHANNEL_POS_OFFSETS[0]] = rb_values.0;
    params[CHANNEL_POS_OFFSETS[1]] = CHANNEL_SIZE;
    params[CHANNEL_POS_OFFSETS[2]] = rb_values.1;
    params[CHANNEL_POS_OFFSETS[3]] = 24;

    let msb_base = (base >> 32) as u32;

    if msb_base != 0 {
        write_u32(params, EXT_LFB_BASE_OFFSET as usize, msb_base);
        write_u32(
            params,
            CAPABILITIES_OFFSET as usize,
            VIDEO_CAPABILITY_64BIT_BASE,
        );
    }
    Ok(())
}
