//! D3D11 Video Processor: RGB→NV12 para o encoder e NV12→RGB para o viewer (decoder por hardware).
use std::mem::ManuallyDrop;

use anyhow::{Context as _, Result, bail};
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_RENDER_TARGET, D3D11_BIND_VIDEO_ENCODER, D3D11_TEX2D_VPIV, D3D11_TEX2D_VPOV,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11_VIDEO_COLOR, D3D11_VIDEO_COLOR_0,
    D3D11_VIDEO_COLOR_RGBA, D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
    D3D11_VIDEO_PROCESSOR_CONTENT_DESC, D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT_OUTPUT,
    D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC, D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0,
    D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC, D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0,
    D3D11_VIDEO_PROCESSOR_STREAM, D3D11_VIDEO_USAGE_PLAYBACK_NORMAL,
    D3D11_VPIV_DIMENSION_TEXTURE2D, D3D11_VPOV_DIMENSION_TEXTURE2D, ID3D11Device,
    ID3D11DeviceContext, ID3D11Texture2D, ID3D11VideoContext, ID3D11VideoContext1,
    ID3D11VideoDevice, ID3D11VideoProcessor, ID3D11VideoProcessorEnumerator,
    ID3D11VideoProcessorOutputView,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709, DXGI_COLOR_SPACE_YCBCR_STUDIO_G22_LEFT_P709,
    DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_NV12, DXGI_FORMAT_R8G8B8A8_UNORM,
    DXGI_RATIONAL, DXGI_SAMPLE_DESC,
};
use windows::core::Interface;

/// Cobre a fila do encoder somada à profundidade do MFT assíncrono.
const POOL: usize = 8;

#[derive(Clone, Copy, PartialEq)]
enum Target {
    Nv12,
    Rgb,
}

pub struct Converter {
    device: ID3D11Device,
    video_device: ID3D11VideoDevice,
    video_context: ID3D11VideoContext,
    video_context1: ID3D11VideoContext1,
    out_w: u32,
    out_h: u32,
    fps: u32,
    src: (u32, u32),
    enumerator: Option<ID3D11VideoProcessorEnumerator>,
    processor: Option<ID3D11VideoProcessor>,
    pool: Vec<(ID3D11Texture2D, ID3D11VideoProcessorOutputView)>,
    next: usize,
    target: Target,
    out_format: DXGI_FORMAT,
}

impl Converter {
    pub fn new(
        device: &ID3D11Device,
        context: &ID3D11DeviceContext,
        out_w: u32,
        out_h: u32,
        fps: u32,
    ) -> Result<Self> {
        Ok(Self {
            device: device.clone(),
            video_device: device.cast().context("ID3D11VideoDevice")?,
            video_context: context.cast().context("ID3D11VideoContext")?,
            video_context1: context.cast().context("ID3D11VideoContext1")?,
            out_w: out_w & !1,
            out_h: out_h & !1,
            fps: fps.max(1),
            src: (0, 0),
            enumerator: None,
            processor: None,
            pool: Vec::new(),
            next: 0,
            target: Target::Nv12,
            out_format: DXGI_FORMAT_NV12,
        })
    }

    /// Saída RGB no tamanho da fonte (viewer): NV12→RGB na GPU.
    pub fn new_rgb(device: &ID3D11Device, context: &ID3D11DeviceContext) -> Result<Self> {
        let mut c = Self::new(device, context, 0, 0, 60)?;
        c.target = Target::Rgb;
        Ok(c)
    }

    /// Formato da textura devolvida por `convert`/`convert_slice`.
    pub fn out_format(&self) -> DXGI_FORMAT {
        self.out_format
    }

    /// Devolve a próxima textura NV12 do pool (round-robin).
    pub fn convert(&mut self, src: &ID3D11Texture2D) -> Result<ID3D11Texture2D> {
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        unsafe { src.GetDesc(&mut desc) };
        if desc.Width == 0 || desc.Height == 0 {
            bail!("quadro de origem vazio");
        }
        self.convert_slice(src, 0, (desc.Width, desc.Height))
    }

    /// `size` é o tamanho visível: na saída do decoder a textura (array) pode ser maior.
    pub fn convert_slice(
        &mut self,
        src: &ID3D11Texture2D,
        slice: u32,
        size: (u32, u32),
    ) -> Result<ID3D11Texture2D> {
        self.ensure(size.0, size.1)?;

        let enumerator = self.enumerator.as_ref().context("enumerador de vídeo")?;
        let processor = self.processor.as_ref().context("processador de vídeo")?;
        let input_desc = D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {
            FourCC: 0,
            ViewDimension: D3D11_VPIV_DIMENSION_TEXTURE2D,
            Anonymous: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 {
                Texture2D: D3D11_TEX2D_VPIV {
                    MipSlice: 0,
                    ArraySlice: slice,
                },
            },
        };
        let mut input = None;
        unsafe {
            self.video_device.CreateVideoProcessorInputView(
                src,
                enumerator,
                &input_desc,
                Some(&mut input),
            )?
        };
        let input = input.context("input view do processador de vídeo")?;

        let index = self.next;
        self.next = (self.next + 1) % self.pool.len();
        let output = self.pool[index].0.clone();

        let mut streams = [D3D11_VIDEO_PROCESSOR_STREAM::default()];
        streams[0].Enable = true.into();
        streams[0].pInputSurface = ManuallyDrop::new(Some(input));
        let res = unsafe {
            self.video_context
                .VideoProcessorBlt(processor, &self.pool[index].1, 0, &streams)
        };
        // `pInputSurface` é ManuallyDrop no crate `windows`: sem soltar aqui a view de entrada
        // (e a textura de captura que ela segura) vaza a cada quadro.
        let _input = unsafe { ManuallyDrop::take(&mut streams[0].pInputSurface) };
        res?;
        Ok(output)
    }

    /// O tamanho de entrada faz parte do content desc, então o processador é recriado quando a fonte muda.
    fn ensure(&mut self, src_w: u32, src_h: u32) -> Result<()> {
        if self.processor.is_some() && self.src == (src_w, src_h) {
            return Ok(());
        }
        if self.target == Target::Rgb {
            self.out_w = src_w & !1;
            self.out_h = src_h & !1;
        }
        let rate = DXGI_RATIONAL {
            Numerator: self.fps,
            Denominator: 1,
        };
        let content = D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
            InputFrameFormat: D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
            InputFrameRate: rate,
            InputWidth: src_w,
            InputHeight: src_h,
            OutputFrameRate: rate,
            OutputWidth: self.out_w,
            OutputHeight: self.out_h,
            Usage: D3D11_VIDEO_USAGE_PLAYBACK_NORMAL,
        };
        let enumerator = unsafe { self.video_device.CreateVideoProcessorEnumerator(&content) }
            .context("criar enumerador de vídeo")?;
        let processor = unsafe { self.video_device.CreateVideoProcessor(&enumerator, 0) }
            .context("criar processador de vídeo")?;

        let view_desc = D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
            ViewDimension: D3D11_VPOV_DIMENSION_TEXTURE2D,
            Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
                Texture2D: D3D11_TEX2D_VPOV { MipSlice: 0 },
            },
        };
        let (format, bind, count) = match self.target {
            Target::Nv12 => (
                DXGI_FORMAT_NV12,
                (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_VIDEO_ENCODER.0) as u32,
                POOL,
            ),
            Target::Rgb => {
                let ok = |f: DXGI_FORMAT| {
                    unsafe { enumerator.CheckVideoProcessorFormat(f) }.is_ok_and(|s| {
                        s & D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT_OUTPUT.0 as u32 != 0
                    })
                };
                let f = [DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_B8G8R8A8_UNORM]
                    .into_iter()
                    .find(|&f| ok(f))
                    .context("processador de vídeo sem saída RGB")?;
                (f, D3D11_BIND_RENDER_TARGET.0 as u32, 1)
            }
        };
        self.out_format = format;
        let mut pool = Vec::with_capacity(count);
        for _ in 0..count {
            let texture = create_output(&self.device, self.out_w, self.out_h, format, bind)?;
            let mut view = None;
            unsafe {
                self.video_device.CreateVideoProcessorOutputView(
                    &texture,
                    &enumerator,
                    &view_desc,
                    Some(&mut view),
                )?
            };
            pool.push((
                texture,
                view.context("output view do processador de vídeo")?,
            ));
        }

        // Estado do processador que só depende do tamanho da fonte: uma vez por (re)criação.
        let source_rect = RECT {
            left: 0,
            top: 0,
            right: src_w as i32,
            bottom: src_h as i32,
        };
        let dest_rect = letterbox(src_w, src_h, self.out_w, self.out_h);
        let black = D3D11_VIDEO_COLOR {
            Anonymous: D3D11_VIDEO_COLOR_0 {
                RGBA: D3D11_VIDEO_COLOR_RGBA {
                    R: 0.0,
                    G: 0.0,
                    B: 0.0,
                    A: 1.0,
                },
            },
        };
        unsafe {
            self.video_context.VideoProcessorSetStreamFrameFormat(
                &processor,
                0,
                D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
            );
            self.video_context.VideoProcessorSetStreamSourceRect(
                &processor,
                0,
                true,
                Some(&source_rect),
            );
            self.video_context.VideoProcessorSetStreamDestRect(
                &processor,
                0,
                true,
                Some(&dest_rect),
            );
            self.video_context
                .VideoProcessorSetOutputBackgroundColor(&processor, false, &black);
            let (stream_cs, out_cs) = match self.target {
                Target::Nv12 => (
                    DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709,
                    DXGI_COLOR_SPACE_YCBCR_STUDIO_G22_LEFT_P709,
                ),
                Target::Rgb => (
                    DXGI_COLOR_SPACE_YCBCR_STUDIO_G22_LEFT_P709,
                    DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709,
                ),
            };
            self.video_context1
                .VideoProcessorSetStreamColorSpace1(&processor, 0, stream_cs);
            self.video_context1
                .VideoProcessorSetOutputColorSpace1(&processor, out_cs);
        }

        self.enumerator = Some(enumerator);
        self.processor = Some(processor);
        self.pool = pool;
        self.src = (src_w, src_h);
        Ok(())
    }
}

fn create_output(
    device: &ID3D11Device,
    w: u32,
    h: u32,
    format: DXGI_FORMAT,
    bind: u32,
) -> Result<ID3D11Texture2D> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: w,
        Height: h,
        MipLevels: 1,
        ArraySize: 1,
        Format: format,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: bind,
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };
    let mut texture = None;
    unsafe { device.CreateTexture2D(&desc, None, Some(&mut texture)) }
        .context("criar textura de saída")?;
    texture.context("textura de saída")
}

/// Retângulo centralizado, com proporção da fonte e coordenadas pares.
fn letterbox(src_w: u32, src_h: u32, out_w: u32, out_h: u32) -> RECT {
    let scale = (out_w as f64 / src_w as f64).min(out_h as f64 / src_h as f64);
    let w = (((src_w as f64 * scale) as u32).min(out_w)) & !1;
    let h = (((src_h as f64 * scale) as u32).min(out_h)) & !1;
    let x = ((out_w - w) / 2) & !1;
    let y = ((out_h - h) / 2) & !1;
    RECT {
        left: x as i32,
        top: y as i32,
        right: (x + w) as i32,
        bottom: (y + h) as i32,
    }
}
