//! Conversão e tone-mapping de fontes HDR para SDR na captura de tela.
//!
//! Monitores com HDR ativado no Windows ('Usar HDR') entregam quadros em scRGB linear
//! (FP16 `DXGI_FORMAT_R16G16B16A16_FLOAT`), onde 1.0 corresponde a 80 nits e valores
//! acima de 1.0 representam realces ou o branco calibrado em 'Brilho do conteúdo SDR'.
//! Se esses quadros forem capturados diretamente como BGRA8 SDR, as cores ficam lavadas
//! ou os realces estouram.
//!
//! Este módulo detecta se a saída está em HDR (`DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020`),
//! lê o nível de branco SDR via `DisplayConfigGetDeviceInfo` (ou fallback seguro de 200 nits)
//! e realiza a conversão na GPU via Pixel Shader D3D11 em um triângulo de tela cheia:
//! 1. Normalização pela escala do branco SDR (nits / 80): o branco SDR vira 1.0.
//! 2. Corte do que passa do branco SDR: o que se compartilha é quase todo conteúdo SDR, e
//!    comprimir realces escureceria o branco das janelas comuns.
//! 3. Conversão de linear para sRGB (gama padrão IEC 61966-2-1).
//! 4. Saída em `DXGI_FORMAT_B8G8R8A8_UNORM` para o FrameSink.

use std::mem::size_of;

use anyhow::{Context as _, Result, anyhow, bail};
use windows::core::{Interface, PCSTR, s};
use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL, DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
    DISPLAYCONFIG_SDR_WHITE_LEVEL, DISPLAYCONFIG_SOURCE_DEVICE_NAME, DisplayConfigGetDeviceInfo,
    GetDisplayConfigBufferSizes, QDC_ONLY_ACTIVE_PATHS, QueryDisplayConfig,
};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D::{D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST, ID3DBlob};
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE,
    D3D11_BUFFER_DESC, D3D11_COMPARISON_NEVER, D3D11_FILTER_MIN_MAG_MIP_POINT, D3D11_FLOAT32_MAX,
    D3D11_SAMPLER_DESC, D3D11_TEXTURE2D_DESC, D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_USAGE_DEFAULT,
    D3D11_VIEWPORT, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11PixelShader,
    ID3D11RenderTargetView, ID3D11SamplerState, ID3D11Texture2D, ID3D11VertexShader,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
    IDXGIDevice, IDXGIFactory1, IDXGIOutput, IDXGIOutput6,
};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, HMONITOR, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST, MonitorFromWindow,
};

use super::SourceId;

/// Nível de branco padrão em nits caso a consulta via DisplayConfig falhe.
/// O Windows costuma calibrar o controle deslizante SDR entre 160 e 240 nits (200 nits nominal);
/// 80 nits é o padrão de sala escura de laboratório e tornaria telas normais excessivamente escuras.
pub const DEFAULT_SDR_WHITE_NITS: f32 = 200.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HdrInfo {
    pub is_hdr: bool,
    pub sdr_white_nits: f32,
}

/// Identifica o HMONITOR associado à fonte selecionada.
pub fn hmonitor_for_source(source: &SourceId) -> Option<isize> {
    match source {
        SourceId::Monitor { hmonitor, .. } => Some(*hmonitor),
        SourceId::Window { hwnd, .. } => unsafe {
            let hmon = MonitorFromWindow(HWND(*hwnd as *mut _), MONITOR_DEFAULTTONEAREST);
            if hmon.is_invalid() {
                None
            } else {
                Some(hmon.0 as isize)
            }
        },
    }
}

/// Localiza a saída DXGI associada ao monitor no dispositivo D3D11 ou no adaptador do sistema.
pub fn find_output_for_monitor(device: &ID3D11Device, hmonitor: isize) -> Option<IDXGIOutput> {
    let dxgi_device: IDXGIDevice = device.cast().ok()?;
    let adapter = unsafe { dxgi_device.GetAdapter() }.ok()?;

    // 1. Tenta saídas do próprio adaptador do device D3D11
    let mut index = 0;
    while let Ok(output) = unsafe { adapter.EnumOutputs(index) } {
        index += 1;
        if let Ok(desc) = unsafe { output.GetDesc() }
            && desc.Monitor.0 as isize == hmonitor
        {
            return Some(output);
        }
    }

    // 2. Se o monitor estiver em outro adaptador (ex.: notebook com GPU integrada e dedicada)
    if let Ok(factory) = unsafe { adapter.GetParent::<IDXGIFactory1>() } {
        let mut adapter_idx = 0;
        while let Ok(other_adapter) = unsafe { factory.EnumAdapters1(adapter_idx) } {
            adapter_idx += 1;
            let mut out_idx = 0;
            while let Ok(out) = unsafe { other_adapter.EnumOutputs(out_idx) } {
                out_idx += 1;
                if let Ok(desc) = unsafe { out.GetDesc() }
                    && desc.Monitor.0 as isize == hmonitor
                {
                    return Some(out);
                }
            }
        }
    }

    None
}

/// Retorna se a saída DXGI está operando em espaço de cores HDR (G2084 / BT.2020).
pub fn is_output_hdr(output: &IDXGIOutput) -> bool {
    let Ok(output6) = output.cast::<IDXGIOutput6>() else {
        return false;
    };
    let Ok(desc1) = (unsafe { output6.GetDesc1() }) else {
        return false;
    };
    desc1.ColorSpace == DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020
}

/// Obtém o nome GDI do monitor (`\\.\DISPLAY1`, etc.) para cruzar com o DisplayConfig.
fn get_monitor_gdi_name(hmonitor: isize) -> Option<[u16; 32]> {
    let mut mi = MONITORINFOEXW::default();
    mi.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
    let ok = unsafe {
        GetMonitorInfoW(
            HMONITOR(hmonitor as *mut _),
            &mut mi.monitorInfo as *mut _,
        )
    };
    ok.as_bool().then_some(mi.szDevice)
}

/// Consulta o nível de branco SDR do Windows para o monitor especificado.
pub fn query_sdr_white_level(hmonitor: isize) -> f32 {
    let gdi_name = get_monitor_gdi_name(hmonitor);
    let mut path_count = 0u32;
    let mut mode_count = 0u32;
    let err = unsafe {
        GetDisplayConfigBufferSizes(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count,
            &mut mode_count,
        )
    };
    if err.0 != 0 || path_count == 0 {
        return DEFAULT_SDR_WHITE_NITS;
    }

    let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
    let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];
    let err = unsafe {
        QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            None,
        )
    };
    if err.0 != 0 {
        return DEFAULT_SDR_WHITE_NITS;
    }

    for path in &paths[..path_count as usize] {
        if let Some(target_gdi) = gdi_name {
            let mut src_name = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
                    size: size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32,
                    adapterId: path.sourceInfo.adapterId,
                    id: path.sourceInfo.id,
                },
                ..Default::default()
            };
            let ret = unsafe {
                DisplayConfigGetDeviceInfo(&mut src_name.header)
            };
            if ret == 0 {
                let matches = src_name
                    .viewGdiDeviceName
                    .iter()
                    .zip(target_gdi.iter())
                    .all(|(&a, &b)| a == b);
                if !matches {
                    continue;
                }
            }
        }

        let mut sdr_level = DISPLAYCONFIG_SDR_WHITE_LEVEL {
            header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL,
                size: size_of::<DISPLAYCONFIG_SDR_WHITE_LEVEL>() as u32,
                adapterId: path.targetInfo.adapterId,
                id: path.targetInfo.id,
            },
            SDRWhiteLevel: 0,
        };
        let ret = unsafe {
            DisplayConfigGetDeviceInfo(&mut sdr_level.header)
        };
        if ret == 0 && sdr_level.SDRWhiteLevel > 0 {
            let nits = (sdr_level.SDRWhiteLevel as f32 / 1000.0) * 80.0;
            if nits > 0.0 {
                return nits;
            }
        }
    }

    DEFAULT_SDR_WHITE_NITS
}

/// Detecta se a fonte está em monitor HDR e extrai suas propriedades.
pub fn detect_hdr(device: &ID3D11Device, source: &SourceId) -> Option<HdrInfo> {
    let hmon = hmonitor_for_source(source)?;
    let output = find_output_for_monitor(device, hmon)?;
    if !is_output_hdr(&output) {
        return None;
    }
    let sdr_white_nits = query_sdr_white_level(hmon);
    Some(HdrInfo {
        is_hdr: true,
        sdr_white_nits,
    })
}

// ============================================================================
// Matemática de Tone-Mapping e sRGB
// ============================================================================

/// Conversão de linear para sRGB padrão (IEC 61966-2-1).
pub fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Mapeia cor scRGB linear para sRGB: o branco SDR do Windows vira 1.0 e realces acima são cortados.
pub fn sc_rgb_to_srgb(rgb: [f32; 3], sdr_white_nits: f32) -> [f32; 3] {
    let scale = (sdr_white_nits / 80.0).max(1.0);
    rgb.map(|c| linear_to_srgb(c / scale))
}

// ============================================================================
// Shader e Conversor GPU D3D11
// ============================================================================

const HLSL_SOURCE: &[u8] = b"
struct VSOutput {
    float4 pos : SV_Position;
    float2 uv : TEXCOORD0;
};

VSOutput vs_main(uint id : SV_VertexID) {
    VSOutput o;
    o.uv = float2((id << 1) & 2, id & 2);
    o.pos = float4(o.uv * float2(2.0f, -2.0f) + float2(-1.0f, 1.0f), 0.0f, 1.0f);
    return o;
}

cbuffer HdrParams : register(b0) {
    float white_scale;
    float3 padding;
};

Texture2D<float4> src_tex : register(t0);
SamplerState sam_point : register(s0);

float linear_to_srgb(float c) {
    c = clamp(c, 0.0f, 1.0f);
    if (c <= 0.0031308f) {
        return c * 12.92f;
    } else {
        return 1.055f * pow(c, 1.0f / 2.4f) - 0.055f;
    }
}

float4 ps_main(VSOutput input) : SV_Target {
    float4 color = src_tex.Sample(sam_point, input.uv);
    float3 n = color.rgb / white_scale;
    float3 srgb = float3(linear_to_srgb(n.r), linear_to_srgb(n.g), linear_to_srgb(n.b));
    return float4(srgb, color.a);
}
";

fn compile_shader(src: &[u8], entry: PCSTR, target: PCSTR) -> Result<ID3DBlob> {
    let mut code = None;
    let mut errors = None;
    let res = unsafe {
        D3DCompile(
            src.as_ptr() as *const _,
            src.len(),
            PCSTR::null(),
            None,
            None,
            entry,
            target,
            0,
            0,
            &mut code,
            Some(&mut errors),
        )
    };
    if let Err(e) = res {
        if let Some(err_blob) = errors {
            let msg = unsafe {
                let ptr = err_blob.GetBufferPointer() as *const u8;
                let len = err_blob.GetBufferSize();
                String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).into_owned()
            };
            bail!("D3DCompile falhou ({target:?}): {e:#} - {msg}");
        }
        bail!("D3DCompile falhou ({target:?}): {e:#}");
    }
    code.ok_or_else(|| anyhow!("D3DCompile retornou código nulo"))
}

pub struct HdrConverter {
    vs: ID3D11VertexShader,
    ps: ID3D11PixelShader,
    cbuffer: ID3D11Buffer,
    sampler: ID3D11SamplerState,
    out_tex: Option<ID3D11Texture2D>,
    rtv: Option<ID3D11RenderTargetView>,
    size: (u32, u32),
    pub sdr_white_nits: f32,
}

impl HdrConverter {
    pub fn new(device: &ID3D11Device, sdr_white_nits: f32) -> Result<Self> {
        let vs_blob = compile_shader(HLSL_SOURCE, s!("vs_main"), s!("vs_4_0"))
            .context("compilar vertex shader HDR->SDR")?;
        let ps_blob = compile_shader(HLSL_SOURCE, s!("ps_main"), s!("ps_4_0"))
            .context("compilar pixel shader HDR->SDR")?;

        let mut vs = None;
        unsafe {
            device.CreateVertexShader(
                std::slice::from_raw_parts(
                    vs_blob.GetBufferPointer() as *const u8,
                    vs_blob.GetBufferSize(),
                ),
                None,
                Some(&mut vs),
            )?;
        }
        let vs = vs.ok_or_else(|| anyhow!("Vertex shader nulo"))?;

        let mut ps = None;
        unsafe {
            device.CreatePixelShader(
                std::slice::from_raw_parts(
                    ps_blob.GetBufferPointer() as *const u8,
                    ps_blob.GetBufferSize(),
                ),
                None,
                Some(&mut ps),
            )?;
        }
        let ps = ps.ok_or_else(|| anyhow!("Pixel shader nulo"))?;

        let cbuffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: 16,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: 0,
        };
        let mut cbuffer = None;
        unsafe { device.CreateBuffer(&cbuffer_desc, None, Some(&mut cbuffer))? };
        let cbuffer = cbuffer.ok_or_else(|| anyhow!("Buffer de constantes nulo"))?;

        let sampler_desc = D3D11_SAMPLER_DESC {
            Filter: D3D11_FILTER_MIN_MAG_MIP_POINT,
            AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
            AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
            AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
            ComparisonFunc: D3D11_COMPARISON_NEVER,
            MinLOD: 0.0,
            MaxLOD: D3D11_FLOAT32_MAX,
            ..Default::default()
        };
        let mut sampler = None;
        unsafe { device.CreateSamplerState(&sampler_desc, Some(&mut sampler))? };
        let sampler = sampler.ok_or_else(|| anyhow!("Sampler state nulo"))?;

        Ok(Self {
            vs,
            ps,
            cbuffer,
            sampler,
            out_tex: None,
            rtv: None,
            size: (0, 0),
            sdr_white_nits,
        })
    }

    /// Atualiza o nível de branco SDR caso as configurações de tela mudem.
    pub fn update_sdr_white_nits(&mut self, sdr_white_nits: f32) {
        self.sdr_white_nits = sdr_white_nits;
    }

    /// Executa a conversão GPU da textura FP16 para a textura BGRA8 SDR de saída.
    pub fn convert(
        &mut self,
        device: &ID3D11Device,
        context: &ID3D11DeviceContext,
        src: &ID3D11Texture2D,
        w: u32,
        h: u32,
    ) -> Result<ID3D11Texture2D> {
        let white_scale = (self.sdr_white_nits / 80.0).max(1.0);
        let cdata = [white_scale, 0.0f32, 0.0f32, 0.0f32];
        unsafe {
            context.UpdateSubresource(
                &self.cbuffer,
                0,
                None,
                cdata.as_ptr() as *const _,
                0,
                0,
            );
        }

        if self.size != (w, h) || self.out_tex.is_none() || self.rtv.is_none() {
            let desc = D3D11_TEXTURE2D_DESC {
                Width: w,
                Height: h,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
                CPUAccessFlags: 0,
                MiscFlags: 0,
            };
            let mut tex = None;
            unsafe { device.CreateTexture2D(&desc, None, Some(&mut tex))? };
            let tex: ID3D11Texture2D =
                tex.ok_or_else(|| anyhow!("Textura de saída HDR->SDR nula"))?;

            let mut rtv = None;
            unsafe { device.CreateRenderTargetView(&tex, None, Some(&mut rtv))? };
            let rtv = rtv.ok_or_else(|| anyhow!("RTV de saída HDR->SDR nulo"))?;

            self.out_tex = Some(tex);
            self.rtv = Some(rtv);
            self.size = (w, h);
        }

        let mut srv = None;
        unsafe { device.CreateShaderResourceView(src, None, Some(&mut srv))? };
        let srv = srv.ok_or_else(|| anyhow!("SRV de entrada HDR nulo"))?;

        let rtv = self.rtv.as_ref().unwrap();
        let out_tex = self.out_tex.as_ref().unwrap().clone();
        let viewport = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: w as f32,
            Height: h as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };

        unsafe {
            context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            context.IASetInputLayout(None);
            context.VSSetShader(&self.vs, None);
            context.PSSetShader(&self.ps, None);
            context.PSSetConstantBuffers(0, Some(&[Some(self.cbuffer.clone())]));
            context.PSSetShaderResources(0, Some(&[Some(srv)]));
            context.PSSetSamplers(0, Some(&[Some(self.sampler.clone())]));
            context.OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
            context.RSSetViewports(Some(&[viewport]));
            context.Draw(3, 0);

            // Desfaz binds para evitar avisos ou travas de recurso do D3D11
            context.OMSetRenderTargets(None, None);
            context.PSSetShaderResources(0, Some(&[None]));
        }

        Ok(out_tex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_to_srgb_properties() {
        assert_eq!(linear_to_srgb(0.0), 0.0);
        assert!((linear_to_srgb(1.0) - 1.0).abs() < 1e-5);

        let mid = linear_to_srgb(0.5);
        // Em sRGB gama ~2.2, 0.5 linear vira ~0.735
        assert!(mid > 0.7 && mid < 0.8);
    }

    #[test]
    fn sc_rgb_to_srgb_keeps_sdr_and_clips_highlights() {
        let near = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5);
        assert!(near(sc_rgb_to_srgb([0.0, 0.0, 0.0], 200.0), [0.0, 0.0, 0.0]));
        // 200 nits → escala 2.5: o branco SDR sai branco, não cinza
        assert!(near(sc_rgb_to_srgb([2.5, 2.5, 2.5], 200.0), [1.0, 1.0, 1.0]));
        // cinza SDR igual ao que seria sem HDR
        let g = sc_rgb_to_srgb([1.25, 1.25, 1.25], 200.0)[0];
        assert!((g - linear_to_srgb(0.5)).abs() < 1e-6);
        // realce acima do branco é cortado; negativo (fora do gamut) vira 0
        assert!(near(sc_rgb_to_srgb([10.0, -1.0, 2.5], 200.0), [1.0, 0.0, 1.0]));
    }
}
