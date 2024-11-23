use std::{
  ffi::{c_void, CStr},
  fs::{create_dir_all, File},
  path::Path,
};

use const_str::cstr;
use formatx::formatx;
use image_webp::{ColorType, WebPEncoder};
use vapours::frame::VapoursVideoFrame;
use vapoursynth4_rs::{
  core::CoreRef,
  declare_plugin,
  frame::{FrameContext, VideoFrame},
  key,
  map::MapRef,
  node::{
    ActivationReason, Dependencies, Filter, FilterDependency, Node, RequestPattern, VideoNode,
  },
  ColorFamily, SampleType,
};

/// WebP encoding as a VapourSynth plugin. Currently does VP8L lossless
/// encoding.
struct WebPFilter {
  /// Input video node.
  node: VideoNode,

  /// Output file path. Supports Rust-like formatting.
  path: String,

  /// If `true`, any missing parents of `path` are created as needed.
  /// Otherwise, a missing parent will throw an error.
  parents: bool,
}

impl Filter for WebPFilter {
  type Error = &'static CStr;
  type FrameType = VideoFrame;
  type FilterData = ();

  fn create(
    input: &MapRef,
    output: &mut MapRef,
    _data: Option<Box<Self::FilterData>>,
    mut core: CoreRef<'_>,
  ) -> Result<(), Self::Error> {
    let Ok(node) = input.get_video_node(key!("clip"), 0) else {
      return Err(cstr!("Failed to get clip."));
    };

    let n = node.clone();
    let vi = n.info();

    if vi.format.bits_per_sample != 8
      || vi.format.color_family != ColorFamily::RGB
      || vi.format.sample_type != SampleType::Integer
    {
      return Err(cstr!("Only RGB24 input is supported."));
    }

    let path = input
      .get_utf8(key!("path"), 0)
      .expect("Missing required `path` parameter.");

    let parents = input.get_int(key!("parents"), 0).unwrap_or(0) == 1;

    let mut filter = Self {
      node,
      path: path.to_owned(),
      parents,
    };

    let deps = [FilterDependency {
      source: filter.node.as_mut_ptr(),
      request_pattern: RequestPattern::StrictSpatial,
    }];

    core.create_video_filter(
      output,
      cstr!("WebP"),
      vi,
      Box::new(filter),
      Dependencies::new(&deps).unwrap(),
    );

    Ok(())
  }

  fn get_frame(
    &self,
    n: i32,
    activation_reason: ActivationReason,
    _frame_data: *mut *mut c_void,
    mut ctx: FrameContext,
    _core: CoreRef<'_>,
  ) -> Result<Option<VideoFrame>, Self::Error> {
    match activation_reason {
      ActivationReason::Initial => {
        ctx.request_frame_filter(n, &self.node);
      }
      ActivationReason::AllFramesReady => {
        let src = self.node.get_frame_filter(n, &mut ctx);
        let r_channel = src.as_slice::<u8>(0);
        let g_channel = src.as_slice::<u8>(1);
        let b_channel = src.as_slice::<u8>(2);
        let width = src.frame_width(0) as usize;
        let height = src.frame_height(0) as usize;

        // Naive RGB packing.
        let mut packed = vec![0; width * height * 3];
        for (((src, r), g), b) in packed
          .chunks_exact_mut(3)
          .zip(r_channel.iter())
          .zip(g_channel.iter())
          .zip(b_channel.iter())
        {
          src[0] = *r;
          src[1] = *g;
          src[2] = *b;
        }

        let formatted_path =
          formatx!(self.path.clone(), n = n).expect("`path` format string was invalid");
        let output_path = Path::new(formatted_path.as_str());

        if self.parents {
          if let Some(parent) = output_path.parent() {
            create_dir_all(parent).expect("Failed to create output directory");
          }
        }

        let encoder = WebPEncoder::new(File::create(output_path).unwrap());
        encoder
          .encode(&packed, width as u32, height as u32, ColorType::Rgb8)
          .expect("Failed to encode to WebP");

        return Ok(Some(src));
      }
      ActivationReason::Error => {}
    }

    Ok(None)
  }

  const NAME: &'static CStr = cstr!("WebP");
  const ARGS: &'static CStr = cstr!("clip:vnode;path:data;parents:int:opt;");
  const RETURN_TYPE: &'static CStr = cstr!("clip:vnode;");
}

declare_plugin!(
  "sgt.webp",
  "webp",
  "WebP encoder.",
  (0, 0),
  VAPOURSYNTH_API_VERSION,
  0,
  (WebPFilter, None)
);
