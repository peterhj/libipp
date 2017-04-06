extern crate icc_link;
extern crate libc;

use ffi::*;

use std::marker::{PhantomData};
//use std::ops::{Deref, DerefMut};
use std::ptr::{null};

pub mod ffi;

pub struct IppBuf<T> where T: Copy {
  ptr:  *mut T,
  len:  usize,
}

impl<T> Drop for IppBuf<T> where T: Copy {
  fn drop(&mut self) {
    assert!(!self.ptr.is_null());
    unsafe { ippsFree(self.ptr as *mut _) };
  }
}

impl IppBuf<u8> {
  pub fn alloc(len: usize) -> IppBuf<u8> {
    let ptr = unsafe { ippsMalloc_8u(len as _) };
    assert!(!ptr.is_null());
    IppBuf{
      ptr:  ptr,
      len:  len,
    }
  }

  pub fn len(&self) -> usize {
    self.len
  }

  pub fn as_ptr(&self) -> *const u8 {
    self.ptr
  }

  pub fn as_mut_ptr(&mut self) -> *mut u8 {
    self.ptr
  }
}

pub fn ipp_copy2d_u8(
    width: usize, height: usize,
    src_offset_x: usize, src_offset_y: usize, src_pitch: usize, src: &[u8],
    dst_offset_x: usize, dst_offset_y: usize, dst_pitch: usize, dst: &mut [u8])
{
  assert!(src_offset_x <= src_pitch);
  assert!(src_pitch * src_offset_y <= src.len());
  assert!(width <= src_pitch);
  assert!(dst_offset_x <= dst_pitch);
  assert!(dst_pitch * dst_offset_y <= dst.len());
  assert!(width <= dst_pitch);
  let src_offset = src_offset_x + src_pitch * src_offset_y;
  let dst_offset = dst_offset_x + dst_pitch * dst_offset_y;
  // TODO(20170217): do more checking to ensure no out of bounds.
  let status = unsafe { ippiCopy_8u_C1R(
      src.as_ptr().offset(src_offset as isize),
      src_pitch as _,
      dst.as_mut_ptr().offset(dst_offset as isize),
      dst_pitch as _,
      IppiSize{width: width as _, height: height as _},
  ) };
  assert!(status.is_ok());
}

pub trait IppImageBufExt<T> where T: Copy {
  fn alloc(width: usize, height: usize) -> Self where Self: Sized;
  fn write(&mut self, ext_buf: &[T]);
  fn write_strided(&mut self, ext_width: usize, ext_height: usize, ext_buf: &[T]);
  fn read(&self, ext_buf: &mut [T]);
  fn read_strided(&self, ext_width: usize, ext_height: usize, ext_buf: &mut [T]);
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

impl IppImageBufExt<u8> for IppImageBuf<u8> {
  fn alloc(width: usize, height: usize) -> IppImageBuf<u8> {
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

  fn write(&mut self, ext_buf: &[u8]) {
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

  fn write_strided(&mut self, ext_width: usize, ext_height: usize, ext_buf: &[u8]) {
    assert!(ext_width <= self.width);
    assert!(ext_height <= self.height);
    assert!(ext_buf.len() <= self.width * self.height);
    let status = unsafe { ippiCopy_8u_C1R(
        ext_buf.as_ptr(),
        ext_width as _,
        self.ptr,
        self.pitch as _,
        IppiSize{width: ext_width as _, height: ext_height as _},
    ) };
    assert!(status.is_ok());
  }

  fn read(&self, ext_buf: &mut [u8]) {
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

  fn read_strided(&self, ext_width: usize, ext_height: usize, ext_buf: &mut [u8]) {
    assert!(ext_width <= self.width);
    assert!(ext_height <= self.height);
    assert!(ext_buf.len() <= self.width * self.height);
    let status = unsafe { ippiCopy_8u_C1R(
        self.ptr,
        self.pitch as _,
        ext_buf.as_mut_ptr(),
        ext_width as _,
        IppiSize{width: ext_width as _, height: ext_height as _},
    ) };
    assert!(status.is_ok());
  }
}

impl IppImageBufExt<f32> for IppImageBuf<f32> {
  fn alloc(width: usize, height: usize) -> IppImageBuf<f32> {
    let mut pitch: i32 = 0;
    let ptr = unsafe { ippiMalloc_32f_C1(width as _, height as _, &mut pitch as *mut _) };
    assert!(!ptr.is_null());
    IppImageBuf{
      ptr:      ptr,
      width:    width,
      height:   height,
      pitch:    pitch as _,
    }
  }

  fn write(&mut self, ext_buf: &[f32]) {
    assert_eq!(ext_buf.len(), self.width * self.height);
    let status = unsafe { ippiCopy_32f_C1R(
        ext_buf.as_ptr(),
        self.width as _,
        self.ptr,
        self.pitch as _,
        IppiSize{width: self.width as _, height: self.height as _},
    ) };
    assert!(status.is_ok());
  }

  fn write_strided(&mut self, ext_width: usize, ext_height: usize, ext_buf: &[f32]) {
    assert!(ext_width <= self.width);
    assert!(ext_height <= self.height);
    assert!(ext_buf.len() <= self.width * self.height);
    let status = unsafe { ippiCopy_32f_C1R(
        ext_buf.as_ptr(),
        ext_width as _,
        self.ptr,
        self.pitch as _,
        IppiSize{width: ext_width as _, height: ext_height as _},
    ) };
    assert!(status.is_ok());
  }

  fn read(&self, ext_buf: &mut [f32]) {
    assert_eq!(ext_buf.len(), self.width * self.height);
    let status = unsafe { ippiCopy_32f_C1R(
        self.ptr,
        self.pitch as _,
        ext_buf.as_mut_ptr(),
        self.width as _,
        IppiSize{width: self.width as _, height: self.height as _},
    ) };
    assert!(status.is_ok());
  }

  fn read_strided(&self, ext_width: usize, ext_height: usize, ext_buf: &mut [f32]) {
    assert!(ext_width <= self.width);
    assert!(ext_height <= self.height);
    assert!(ext_buf.len() <= self.width * self.height);
    let status = unsafe { ippiCopy_32f_C1R(
        self.ptr,
        self.pitch as _,
        ext_buf.as_mut_ptr(),
        ext_width as _,
        IppiSize{width: ext_width as _, height: ext_height as _},
    ) };
    assert!(status.is_ok());
  }
}

#[derive(Clone, Copy)]
pub enum IppImageResizeKind {
  Linear,
  Cubic{b: f32, c: f32},
  Lanczos{nlobes: usize},
}

pub trait IppImageResizeExt<T> where T: Copy {
  fn create(kind: IppImageResizeKind, src_width: usize, src_height: usize, dst_width: usize, dst_height: usize) -> Result<Self, ()> where Self: Sized;
  fn resize(&mut self, src: &IppImageBuf<T>, dst: &mut IppImageBuf<T>);
}

pub struct IppImageResize<T> where T: Copy {
  spec: IppBuf<u8>,
  //bord: IppBuf<u8>,
  buf:  IppBuf<u8>,
  kind: IppImageResizeKind,
  src:  (usize, usize),
  dst:  (usize, usize),
  _mrk: PhantomData<fn (T)>,
}

impl IppImageResizeExt<u8> for IppImageResize<u8> {
  fn create(kind: IppImageResizeKind, src_width: usize, src_height: usize, dst_width: usize, dst_height: usize) -> Result<Self, ()> {
    let interp_ty = match kind {
      IppImageResizeKind::Linear        => IppiInterpolationType::ippLinear,
      IppImageResizeKind::Cubic{..}     => IppiInterpolationType::ippCubic,
      IppImageResizeKind::Lanczos{..}   => IppiInterpolationType::ippLanczos,
    };
    let mut spec_size = 0;
    let mut init_buf_size = 0;
    let status = unsafe { ippiResizeGetSize_8u(
        IppiSize{width: src_width as _, height: src_height as _},
        IppiSize{width: dst_width as _, height: dst_height as _},
        interp_ty,
        0, // antialiasing.
        &mut spec_size as *mut _,
        &mut init_buf_size as *mut _,
    ) };
    assert!(status.is_ok());
    let mut spec = IppBuf::<u8>::alloc(spec_size as _);
    match kind {
      IppImageResizeKind::Linear => {
        let status = unsafe { ippiResizeLinearInit_8u(
            IppiSize{width: src_width as _, height: src_height as _},
            IppiSize{width: dst_width as _, height: dst_height as _},
            spec.as_mut_ptr() as *mut _,
        ) };
        assert!(status.is_ok());
      }
      IppImageResizeKind::Cubic{b, c} => {
        let mut init_buf = IppBuf::<u8>::alloc(init_buf_size as _);
        let status = unsafe { ippiResizeCubicInit_8u(
            IppiSize{width: src_width as _, height: src_height as _},
            IppiSize{width: dst_width as _, height: dst_height as _},
            b, c,
            spec.as_mut_ptr() as *mut _,
            init_buf.as_mut_ptr(),
        ) };
        assert!(status.is_ok());
      }
      IppImageResizeKind::Lanczos{nlobes} => {
        let mut init_buf = IppBuf::<u8>::alloc(init_buf_size as _);
        let status = unsafe { ippiResizeLanczosInit_8u(
            IppiSize{width: src_width as _, height: src_height as _},
            IppiSize{width: dst_width as _, height: dst_height as _},
            nlobes as _,
            spec.as_mut_ptr() as *mut _,
            init_buf.as_mut_ptr(),
        ) };
        assert!(status.is_ok());
      }
    }
    /*let mut border_size = IppiBorderSize::default();
    let status = unsafe { ippiResizeGetBorderSize_8u(
        spec.as_ptr() as *const IppiResizeSpec_32f,
        &mut border_size as *mut _,
    ) };
    assert!(status.is_ok());
    let border_circum = border_size.border_left + border_size.border_top + border_size.border_right + border_size.border_bottom;
    let bord = IppBuf::<u8>::alloc(border_circum as _);*/
    let mut buf_size = 0;
    let status = unsafe { ippiResizeGetBufferSize_8u(
        spec.as_ptr() as *const IppiResizeSpec_32f,
        IppiSize{width: dst_width as _, height: dst_height as _},
        1, // num channels.
        &mut buf_size as *mut _,
    ) };
    assert!(status.is_ok());
    let buf = IppBuf::<u8>::alloc(buf_size as _);
    Ok(IppImageResize{
      spec: spec,
      //bord: bord,
      buf:  buf,
      kind: kind,
      src:  (src_width, src_height),
      dst:  (dst_width, dst_height),
      _mrk: PhantomData,
    })
  }

  fn resize(&mut self, src: &IppImageBuf<u8>, dst: &mut IppImageBuf<u8>) {
    assert!(self.src.0 <= src.width);
    assert!(self.src.1 <= src.height);
    assert!(self.dst.0 <= dst.width);
    assert!(self.dst.1 <= dst.height);
    /*println!("DEBUG: ipp: resize: {} x {} ({}) -> {} x {} ({})",
        src.width, src.height, src.pitch, dst.width, dst.height, dst.pitch);*/
    match self.kind {
      IppImageResizeKind::Linear        => {
        let status = unsafe { ippiResizeLinear_8u_C1R(
            src.ptr,
            src.pitch as _,
            dst.ptr,
            dst.pitch as _,
            IppiPoint{x: 0, y: 0},
            IppiSize{width: self.dst.0 as _, height: self.dst.1 as _},
            IppiBorderType::ippBorderRepl,
            null(),
            self.spec.as_ptr() as *const IppiResizeSpec_32f,
            self.buf.as_mut_ptr(),
        ) };
        assert!(status.is_ok());
      }
      IppImageResizeKind::Cubic{..}     => {
        let status = unsafe { ippiResizeCubic_8u_C1R(
            src.ptr,
            src.pitch as _,
            dst.ptr,
            dst.pitch as _,
            IppiPoint{x: 0, y: 0},
            IppiSize{width: self.dst.0 as _, height: self.dst.1 as _},
            IppiBorderType::ippBorderRepl,
            null(),
            self.spec.as_ptr() as *const IppiResizeSpec_32f,
            self.buf.as_mut_ptr(),
        ) };
        assert!(status.is_ok());
      }
      IppImageResizeKind::Lanczos{..}   => {
        let status = unsafe { ippiResizeLanczos_8u_C1R(
            src.ptr,
            src.pitch as _,
            dst.ptr,
            dst.pitch as _,
            IppiPoint{x: 0, y: 0},
            IppiSize{width: self.dst.0 as _, height: self.dst.1 as _},
            IppiBorderType::ippBorderRepl,
            null(),
            self.spec.as_ptr() as *const IppiResizeSpec_32f,
            self.buf.as_mut_ptr(),
        ) };
        assert!(status.is_ok());
      }
    }
  }
}

impl IppImageResizeExt<f32> for IppImageResize<f32> {
  fn create(kind: IppImageResizeKind, src_width: usize, src_height: usize, dst_width: usize, dst_height: usize) -> Result<Self, ()> {
    let interp_ty = match kind {
      IppImageResizeKind::Linear        => IppiInterpolationType::ippLinear,
      IppImageResizeKind::Cubic{..}     => IppiInterpolationType::ippCubic,
      IppImageResizeKind::Lanczos{..}   => IppiInterpolationType::ippLanczos,
    };
    let mut spec_size = 0;
    let mut init_buf_size = 0;
    let status = unsafe { ippiResizeGetSize_32f(
        IppiSize{width: src_width as _, height: src_height as _},
        IppiSize{width: dst_width as _, height: dst_height as _},
        interp_ty,
        0, // antialiasing.
        &mut spec_size as *mut _,
        &mut init_buf_size as *mut _,
    ) };
    assert!(status.is_ok());
    let mut spec = IppBuf::<u8>::alloc(spec_size as _);
    match kind {
      IppImageResizeKind::Linear => {
        let status = unsafe { ippiResizeLinearInit_32f(
            IppiSize{width: src_width as _, height: src_height as _},
            IppiSize{width: dst_width as _, height: dst_height as _},
            spec.as_mut_ptr() as *mut _,
        ) };
        assert!(status.is_ok());
      }
      IppImageResizeKind::Cubic{b, c} => {
        let mut init_buf = IppBuf::<u8>::alloc(init_buf_size as _);
        let status = unsafe { ippiResizeCubicInit_32f(
            IppiSize{width: src_width as _, height: src_height as _},
            IppiSize{width: dst_width as _, height: dst_height as _},
            b, c,
            spec.as_mut_ptr() as *mut _,
            init_buf.as_mut_ptr(),
        ) };
        assert!(status.is_ok());
      }
      IppImageResizeKind::Lanczos{nlobes} => {
        let mut init_buf = IppBuf::<u8>::alloc(init_buf_size as _);
        let status = unsafe { ippiResizeLanczosInit_32f(
            IppiSize{width: src_width as _, height: src_height as _},
            IppiSize{width: dst_width as _, height: dst_height as _},
            nlobes as _,
            spec.as_mut_ptr() as *mut _,
            init_buf.as_mut_ptr(),
        ) };
        assert!(status.is_ok());
      }
    }
    let mut buf_size = 0;
    let status = unsafe { ippiResizeGetBufferSize_32f(
        spec.as_ptr() as *const IppiResizeSpec_32f,
        IppiSize{width: dst_width as _, height: dst_height as _},
        1, // num channels.
        &mut buf_size as *mut _,
    ) };
    assert!(status.is_ok());
    let buf = IppBuf::<u8>::alloc(buf_size as _);
    Ok(IppImageResize{
      spec: spec,
      buf:  buf,
      kind: kind,
      src:  (src_width, src_height),
      dst:  (dst_width, dst_height),
      _mrk: PhantomData,
    })
  }

  fn resize(&mut self, src: &IppImageBuf<f32>, dst: &mut IppImageBuf<f32>) {
    assert!(self.src.0 <= src.width);
    assert!(self.src.1 <= src.height);
    assert!(self.dst.0 <= dst.width);
    assert!(self.dst.1 <= dst.height);
    match self.kind {
      IppImageResizeKind::Linear => {
        let status = unsafe { ippiResizeLinear_32f_C1R(
            src.ptr,
            src.pitch as _,
            dst.ptr,
            dst.pitch as _,
            IppiPoint{x: 0, y: 0},
            IppiSize{width: self.dst.0 as _, height: self.dst.1 as _},
            IppiBorderType::ippBorderRepl,
            null(),
            self.spec.as_ptr(),
            self.buf.as_mut_ptr(),
        ) };
        assert!(status.is_ok());
      }
      IppImageResizeKind::Cubic{..} => {
        let status = unsafe { ippiResizeCubic_32f_C1R(
            src.ptr,
            src.pitch as _,
            dst.ptr,
            dst.pitch as _,
            IppiPoint{x: 0, y: 0},
            IppiSize{width: self.dst.0 as _, height: self.dst.1 as _},
            IppiBorderType::ippBorderRepl,
            null(),
            self.spec.as_ptr(),
            self.buf.as_mut_ptr(),
        ) };
        assert!(status.is_ok());
      }
      IppImageResizeKind::Lanczos{..} => {
        let status = unsafe { ippiResizeLanczos_32f_C1R(
            src.ptr,
            src.pitch as _,
            dst.ptr,
            dst.pitch as _,
            IppiPoint{x: 0, y: 0},
            IppiSize{width: self.dst.0 as _, height: self.dst.1 as _},
            IppiBorderType::ippBorderRepl,
            null(),
            self.spec.as_ptr(),
            self.buf.as_mut_ptr(),
        ) };
        assert!(status.is_ok());
      }
    }
  }
}

pub struct IppImageDownsamplePyramid<T> where T: Copy {
  bufs: Vec<IppImageBuf<T>>,
  ops:  Vec<IppImageResize<T>>,
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
    while prev_width > dst_width || prev_height > dst_height {
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
      /*println!("DEBUG: ipp: pyramid: level: {} x {} -> {} x {}",
          prev_width, prev_height, next_width, next_height);*/
      bufs.push(IppImageBuf::<u8>::alloc(next_width, next_height));
      ops.push(IppImageResize::<u8>::create(IppImageResizeKind::Linear, prev_width, prev_height, next_width, next_height).unwrap());
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
    self.bufs[0].write(src);
    for k in 0 .. num_levels {
      let (prev_bufs, mut next_bufs) = self.bufs.split_at_mut(k+1);
      self.ops[k].resize(&prev_bufs[k], &mut next_bufs[0]);
    }
    self.bufs[num_levels].read(dst);
  }
}
