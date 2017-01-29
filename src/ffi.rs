#![allow(non_camel_case_types)]

use libc::*;

//pub struct IppStatus(pub c_int);

#[derive(Clone, Copy)]
#[repr(C)]
pub enum IppStatus {
  IppStsNoErr = 0,
  // TODO
}

impl IppStatus {
  pub fn is_ok(self) -> bool {
    self as i32 == 0
  }

  pub fn is_err(self) -> bool {
    !self.is_ok()
  }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct IppiRect {
  pub x:        c_int,
  pub y:        c_int,
  pub width:    c_int,
  pub height:   c_int,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct IppiPoint {
  pub x:        c_int,
  pub y:        c_int,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct IppiSize {
  pub width:    c_int,
  pub height:   c_int,
}

/*#[repr(C)]
pub enum IppiResizeFilterType {
}

pub enum IppiResizeFilterState {}*/

#[derive(Clone, Copy)]
#[repr(C)]
pub enum IppiInterpolationType {
  ippNearest    = 1,
  ippLinear     = 2,
  ippCubic      = 6,
  ippLanczos    = 16,
  ippHahn       = 0,
  ippSuper      = 8,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct IppiBorderSize {
  pub border_left:      u32,
  pub border_top:       u32,
  pub border_right:     u32,
  pub border_bottom:    u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub enum IppiBorderType {
  ippBorderRepl         = 1,
  ippBorderWrap         = 2,
  ippBorderMirror       = 3,
  ippBorderMirrorR      = 4,
  ippBorderDefault      = 5,
  ippBorderConst        = 6,
  ippBorderTransp       = 7,
  ippBorderInMemTop     = 0x0010,
  ippBorderInMemBottom  = 0x0020,
  ippBorderInMemLeft    = 0x0040,
  ippBorderInMemRight   = 0x0080,
  ippBorderInMem        = 0x00f0,
}

pub type IppiResizeSpec_32f = u8;

#[link(name = "ippcore")]
extern "C" {
}

#[link(name = "ipps")]
extern "C" {
  pub fn ippsMalloc_8u(size: c_int) -> *mut u8;
  pub fn ippsFree(ptr: *mut c_void);
}

#[link(name = "ippi")]
extern "C" {
  pub fn ippiMalloc_8u_C1(width_pixels: c_int, height_pixels: c_int, pitch: *mut c_int) -> *mut u8;
  pub fn ippiFree(ptr: *mut c_void);

  pub fn ippiCopy_8u_C1R(src: *const u8, src_pitch: c_int, dst: *mut u8, dst_pitch: c_int, roi_size: IppiSize) -> IppStatus;

  pub fn ippiResizeGetSize_8u(src_size: IppiSize, dst_size: IppiSize, interpolation: IppiInterpolationType, antialiasing: u32, spec_size: *mut c_int, init_buf_size: *mut c_int) -> IppStatus;
  pub fn ippiResizeLinearInit_8u(src_size: IppiSize, dst_size: IppiSize, spec: *mut IppiResizeSpec_32f) -> IppStatus;
  pub fn ippiResizeGetBufferSize_8u(spec: *const IppiResizeSpec_32f, dst_size: IppiSize, num_channels: u32, buf_size: *mut c_int) -> IppStatus;
  pub fn ippiResizeGetBorderSize_8u(spec: *const IppiResizeSpec_32f, border_size: *mut IppiBorderSize) -> IppStatus;
  pub fn ippiResizeLinear_8u_C1R(src: *const u8, src_pitch: i32, dst: *mut u8, dst_pitch: i32, dst_offset: IppiPoint, dst_size: IppiSize, border: IppiBorderType, border_value: *const u8, spec: *const IppiResizeSpec_32f, buf: *mut u8) -> IppStatus;
}
