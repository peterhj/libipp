extern crate icc_link;
extern crate libc;

use ffi::*;

use std::marker::{PhantomData};
//use std::ops::{Deref, DerefMut};

pub mod ffi;

pub struct IppTemporalBuf<T> where T: Copy {
  ptr:      *mut T,
  _size:    usize,
}

impl<T> Drop for IppTemporalBuf<T> where T: Copy {
  fn drop(&mut self) {
    assert!(!self.ptr.is_null());
    unsafe { ippsFree(self.ptr as *mut _) };
  }
}

impl IppTemporalBuf<u8> {
  pub fn alloc(size: usize) -> IppTemporalBuf<u8> {
    let ptr = unsafe { ippsMalloc_8u(size as _) };
    assert!(!ptr.is_null());
    IppTemporalBuf{
      ptr:      ptr,
      _size:    size,
    }
  }

  pub fn as_ptr(&self) -> *const u8 {
    self.ptr
  }

  pub fn as_mut_ptr(&mut self) -> *mut u8 {
    self.ptr
  }
}

pub struct IppImageBuf<T> where T: Copy {
  ptr:      *mut T,
  width:    usize,
  height:   usize,
  pitch:    usize,
}

impl<T> Drop for IppImageBuf<T> where T: Copy {
  fn drop(&mut self) {
    assert!(!self.ptr.is_null());
    unsafe { ippiFree(self.ptr as *mut _) };
  }
}

impl IppImageBuf<u8> {
  pub fn alloc(width: usize, height: usize) -> IppImageBuf<u8> {
    let mut pitch: i32 = 0;
    let ptr = unsafe { ippiMalloc_8u_C1(width as _, height as _, &mut pitch as *mut _) };
    assert!(!ptr.is_null());
    IppImageBuf{
      ptr:      ptr,
      width:    width,
      height:   height,
      pitch:    pitch as _,
    }
  }

  pub fn load(&mut self, ext_buf: &[u8]) {
    assert_eq!(ext_buf.len(), self.width * self.height);
    let status = unsafe { ippiCopy_8u_C1R(
        ext_buf.as_ptr(),
        self.width as _,
        self.ptr,
        self.pitch as _,
        IppiSize{width: self.width as _, height: self.height as _},
    ) };
    assert!(status.is_ok());
  }

  pub fn store(&self, ext_buf: &mut [u8]) {
    assert_eq!(ext_buf.len(), self.width * self.height);
    let status = unsafe { ippiCopy_8u_C1R(
        self.ptr,
        self.pitch as _,
        ext_buf.as_mut_ptr(),
        self.width as _,
        IppiSize{width: self.width as _, height: self.height as _},
    ) };
    assert!(status.is_ok());
  }
}

pub struct IppImageResizeLinear<T> where T: Copy {
  spec: IppTemporalBuf<u8>,
  buf:  IppTemporalBuf<u8>,
  src:  (usize, usize),
  dst:  (usize, usize),
  _mrk: PhantomData<fn (T)>,
}

impl IppImageResizeLinear<u8> {
  pub fn new(src_width: usize, src_height: usize, dst_width: usize, dst_height: usize) -> Self {
    let mut spec_size = 0;
    let mut init_buf_size = 0;
    let status = unsafe { ippiResizeGetSize_8u(
        IppiSize{width: src_width as _, height: src_height as _},
        IppiSize{width: dst_width as _, height: dst_height as _},
        IppiInterpolationType::ippLinear,
        0, // antialiasing.
        &mut spec_size as *mut _,
        &mut init_buf_size as *mut _,
    ) };
    assert!(status.is_ok());
    assert_eq!(0, init_buf_size);
    let mut spec = IppTemporalBuf::<u8>::alloc(spec_size as _);
    let status = unsafe { ippiResizeLinearInit_8u(
        IppiSize{width: src_width as _, height: src_height as _},
        IppiSize{width: dst_width as _, height: dst_height as _},
        spec.as_mut_ptr() as *mut _,
    ) };
    assert!(status.is_ok());
    let mut buf_size = 0;
    let status = unsafe { ippiResizeGetBufferSize_8u(
        spec.as_ptr() as *const IppiResizeSpec_32f,
        IppiSize{width: dst_width as _, height: dst_height as _},
        1, // num channels.
        &mut buf_size as *mut _,
    ) };
    assert!(status.is_ok());
    let buf = IppTemporalBuf::<u8>::alloc(buf_size as _);
    IppImageResizeLinear{
      spec: spec,
      buf:  buf,
      src:  (src_width, src_height),
      dst:  (dst_width, dst_height),
      _mrk: PhantomData,
    }
  }

  pub fn resize(&mut self, src: &IppImageBuf<u8>, dst: &mut IppImageBuf<u8>) {
    assert_eq!(self.src.0, src.width);
    assert_eq!(self.src.1, src.height);
    assert_eq!(self.dst.0, dst.width);
    assert_eq!(self.dst.1, dst.height);
    let status = unsafe { ippiResizeLinear_8u_C1R(
        src.ptr,
        src.pitch as _,
        dst.ptr,
        dst.pitch as _,
        IppiPoint{x: 0, y: 0},
        IppiSize{width: dst.width as _, height: dst.height as _},
        self.spec.as_ptr() as *const IppiResizeSpec_32f,
        self.buf.as_mut_ptr(),
    ) };
    assert!(status.is_ok());
  }
}

pub struct IppImageDownsamplePyramid<T> where T: Copy {
  bufs: Vec<IppImageBuf<T>>,
  ops:  Vec<IppImageResizeLinear<T>>,
  src:  (usize, usize),
  dst:  (usize, usize),
}

impl IppImageDownsamplePyramid<u8> {
  pub fn new(src_width: usize, src_height: usize, dst_width: usize, dst_height: usize) -> Self {
    assert!(src_width >= dst_width);
    assert!(src_height >= dst_height);
    let mut bufs = vec![];
    let mut ops = vec![];
    bufs.push(IppImageBuf::<u8>::alloc(src_width, src_height));
    let mut prev_width = src_width;
    let mut prev_height = src_height;
    while prev_width > dst_width && prev_height > dst_height {
      let next_width = if prev_width >= 2 * dst_width {
        (prev_width + 1) / 2
      } else {
        dst_width
      };
      let next_height = if prev_height >= 2 * dst_height {
        (prev_height + 1) / 2
      } else {
        dst_height
      };
      bufs.push(IppImageBuf::<u8>::alloc(next_width, next_height));
      ops.push(IppImageResizeLinear::<u8>::new(prev_width, prev_height, next_width, next_height));
      prev_width = next_width;
      prev_height = next_height;
    }
    IppImageDownsamplePyramid{
      bufs: bufs,
      ops:  ops,
      src:  (src_width, src_height),
      dst:  (dst_width, dst_height),
    }
  }

  pub fn downsample(&mut self, src: &[u8], dst: &mut [u8]) {
    assert_eq!(self.src.0 * self.src.1, src.len());
    assert_eq!(self.dst.0 * self.dst.1, dst.len());
    let num_levels = self.ops.len();
    self.bufs[0].load(src);
    for k in 0 .. num_levels {
      let (prev_bufs, mut next_bufs) = self.bufs.split_at_mut(k+1);
      self.ops[k].resize(&prev_bufs[k], &mut next_bufs[0]);
    }
    self.bufs[num_levels].store(dst);
  }
}
