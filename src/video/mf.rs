//! Helpers do Media Foundation compartilhados pelo encoder, decoder e probe.
use std::mem::ManuallyDrop;
use std::sync::Once;

use anyhow::{Result, anyhow};
use tracing::{debug, warn};
use windows::Win32::Foundation::VARIANT_BOOL;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoTaskMemFree};
use windows::Win32::System::Variant::{VARENUM, VARIANT, VARIANT_0, VARIANT_0_0, VT_BOOL, VT_UI4};
use windows::core::GUID;

/// Inicializa COM (MTA) na thread atual e o Media Foundation uma única vez por processo.
///
/// COM é por thread: cada thread que cria MFTs precisa do seu próprio apartamento,
/// então `CoInitializeEx` roda a cada chamada (barato e idempotente).
pub fn startup() {
    static MF: Once = Once::new();
    unsafe {
        // tolera "já inicializado" (S_FALSE) e apartamento STA já existente (RPC_E_CHANGED_MODE)
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    MF.call_once(|| unsafe {
        if let Err(e) = MFStartup(MF_VERSION, MFSTARTUP_FULL) {
            warn!("MFStartup falhou: {e}");
        }
    });
}

/// Percorre start codes Annex B e verifica se algum NAL é SPS (tipo 7).
pub fn has_sps(annexb: &[u8]) -> bool {
    let mut i = 0;
    while i + 3 < annexb.len() {
        if annexb[i] == 0 && annexb[i + 1] == 0 && annexb[i + 2] == 1 {
            if annexb[i + 3] & 0x1f == 7 {
                return true;
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    false
}

pub fn prepend_sequence_header(header: &[u8], frame: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(header.len() + frame.len());
    v.extend_from_slice(header);
    v.extend_from_slice(frame);
    v
}

/// MF_MT_FRAME_SIZE / MF_MT_FRAME_RATE: primeiro valor nos 32 bits baixos.
pub fn pack_pair(a: u32, b: u32) -> u64 {
    ((a as u64) << 32) | b as u64
}

pub fn unpack_pair(v: u64) -> (u32, u32) {
    ((v >> 32) as u32, v as u32)
}

/// Monta um `VARIANT` com `vt` e um valor preenchido pelo chamador.
fn variant_with(
    vt: VARENUM,
    f: impl FnOnce(&mut windows::Win32::System::Variant::VARIANT_0_0_0),
) -> VARIANT {
    let mut inner = VARIANT_0_0::default();
    inner.vt = vt;
    f(&mut inner.Anonymous);
    let mut var = VARIANT::default();
    var.Anonymous = VARIANT_0 {
        Anonymous: std::mem::ManuallyDrop::new(inner),
    };
    var
}

pub fn variant_u32(v: u32) -> VARIANT {
    variant_with(VT_UI4, |x| x.ulVal = v)
}

pub fn variant_bool(v: bool) -> VARIANT {
    variant_with(VT_BOOL, |x| {
        x.boolVal = VARIANT_BOOL(if v { -1 } else { 0 })
    })
}

/// Enumera MFTs da categoria. O array devolvido por `MFTEnumEx` é liberado com
/// `CoTaskMemFree`; as referências `IMFActivate` passam a pertencer ao Vec.
pub fn enumerate_mfts(
    category: GUID,
    flags: MFT_ENUM_FLAG,
    input: Option<MFT_REGISTER_TYPE_INFO>,
    output: Option<MFT_REGISTER_TYPE_INFO>,
) -> Vec<IMFActivate> {
    let mut arr: *mut Option<IMFActivate> = std::ptr::null_mut();
    let mut n = 0u32;
    let hr = unsafe {
        MFTEnumEx(
            category,
            flags,
            input.as_ref().map(|i| i as *const _),
            output.as_ref().map(|o| o as *const _),
            &mut arr,
            &mut n,
        )
    };
    if hr.is_err() || arr.is_null() {
        if let Err(e) = hr {
            debug!("MFTEnumEx falhou: {e}");
        }
        return Vec::new();
    }
    let mut v = Vec::with_capacity(n as usize);
    for i in 0..n as usize {
        // transfere a posse do ponteiro COM para o Vec
        if let Some(a) = unsafe { std::ptr::read(arr.add(i)) } {
            v.push(a);
        }
    }
    unsafe { CoTaskMemFree(Some(arr as *const _)) };
    v
}

pub fn friendly_name(activate: &IMFActivate) -> String {
    let n = unsafe { activate.GetStringLength(&MFT_FRIENDLY_NAME_Attribute) }.unwrap_or(0);
    if n == 0 {
        return String::new();
    }
    let mut buf = vec![0u16; n as usize + 1];
    let mut len = 0u32;
    if unsafe { activate.GetString(&MFT_FRIENDLY_NAME_Attribute, &mut buf, Some(&mut len)) }
        .is_err()
    {
        return String::new();
    }
    let len = (len as usize).min(buf.len());
    // `GetString` devolve o tamanho incluindo o terminador quando ele cabe.
    String::from_utf16_lossy(&buf[..len])
        .trim_end_matches('\0')
        .to_string()
}

pub fn is_async(t: &IMFTransform) -> bool {
    unsafe {
        t.GetAttributes()
            .ok()
            .and_then(|a| a.GetUINT32(&MF_TRANSFORM_ASYNC).ok())
            == Some(1)
    }
}

pub enum OutputResult {
    Sample(IMFSample),
    NeedMoreInput,
    StreamChange,
}

/// Sample de saída reaproveitado entre chamadas (o MFT não fornece os seus).
#[derive(Default)]
pub struct OutSample {
    sample: Option<IMFSample>,
    len: u32,
}

impl OutSample {
    fn give_back(&mut self, sample: &Option<IMFSample>) {
        if let Some(s) = sample {
            self.sample = Some(s.clone());
        }
    }
}

/// Um `ProcessOutput`. Aloca o sample quando o MFT não fornece os seus; `spare`
/// guarda o sample anterior para não alocar um buffer novo a cada quadro.
///
/// O chamador não pode manter o sample devolvido entre duas chamadas: o mesmo
/// buffer é reaproveitado na chamada seguinte.
pub fn process_output(t: &IMFTransform, spare: &mut OutSample) -> Result<OutputResult> {
    let mut buf = [MFT_OUTPUT_DATA_BUFFER::default()];
    let mut ours = false;
    let info = unsafe { t.GetOutputStreamInfo(0) }.ok();
    if let Some(info) =
        info.filter(|i| i.dwFlags & MFT_OUTPUT_STREAM_PROVIDES_SAMPLES.0 as u32 == 0)
    {
        ours = true;
        // shortcut: cbSize desconhecido vira 4 MiB; MFTs de hardware fornecem os próprios samples
        let len = info.cbSize.max(4 * 1024 * 1024);
        if spare.len < len || spare.sample.is_none() {
            // sem cache, ou buffer menor que o pedido atual (mudou a resolução)
            spare.sample = None;
            spare.len = len;
        }
        let cached = spare.sample.take();
        let sample = match cached {
            Some(s) => {
                // estado residual do quadro anterior (atributos, tamanho, tempos) faz alguns MFTs falharem
                unsafe {
                    let _ = windows::core::Interface::cast::<IMFAttributes>(&s)
                        .and_then(|a| a.DeleteAllItems());
                    let _ = s.SetSampleTime(0);
                    let _ = s.SetSampleDuration(0);
                    if let Ok(b) = s.GetBufferByIndex(0) {
                        let _ = b.SetCurrentLength(0);
                    }
                }
                s
            }
            None => unsafe {
                let sample = MFCreateSample()?;
                let mb = MFCreateMemoryBuffer(spare.len)?;
                sample.AddBuffer(&mb)?;
                sample
            },
        };
        buf[0].pSample = ManuallyDrop::new(Some(sample));
    }
    let mut status = 0u32;
    let res = unsafe { t.ProcessOutput(0, &mut buf, &mut status) };
    let sample = unsafe { ManuallyDrop::take(&mut buf[0].pSample) };
    // eventos do MFT, se houver, precisam ser liberados
    let _events = unsafe { ManuallyDrop::take(&mut buf[0].pEvents) };
    // o sample é nosso em qualquer resultado (inclusive NeedMoreInput, o fim normal de
    // todo drain): devolve ao cache para o próximo quadro não alocar de novo
    if ours {
        spare.give_back(&sample);
    }
    match res {
        Ok(()) => match sample {
            Some(s) => Ok(OutputResult::Sample(s)),
            None => Err(anyhow!("MFT não entregou sample")),
        },
        Err(e) => match e.code() {
            MF_E_TRANSFORM_NEED_MORE_INPUT => Ok(OutputResult::NeedMoreInput),
            MF_E_TRANSFORM_STREAM_CHANGE => Ok(OutputResult::StreamChange),
            _ => Err(e.into()),
        },
    }
}

pub fn sample_bytes(sample: &IMFSample) -> Result<Vec<u8>> {
    let buf = unsafe { sample.ConvertToContiguousBuffer()? };
    let mut ptr: *mut u8 = std::ptr::null_mut();
    let mut max = 0u32;
    let mut cur = 0u32;
    unsafe { buf.Lock(&mut ptr, Some(&mut max), Some(&mut cur))? };
    // sem zerar: o copy abaixo preenche o buffer inteiro
    let mut v = Vec::with_capacity(cur as usize);
    if cur > 0 && !ptr.is_null() {
        unsafe { v.extend_from_slice(std::slice::from_raw_parts(ptr, cur as usize)) };
    }
    unsafe { buf.Unlock()? };
    Ok(v)
}

/// `ICodecAPI::SetValue` só quando suportado; não suportado não é erro. `true` se aplicou.
pub fn set_codec_value(api: &ICodecAPI, guid: &GUID, value: VARIANT) -> bool {
    unsafe {
        if api.IsSupported(guid).is_err() {
            debug!("codec api não suportada: {guid:?}");
            return false;
        }
        if let Err(e) = api.SetValue(guid, &value) {
            debug!("SetValue({guid:?}) falhou: {e}");
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sps() {
        assert!(has_sps(&[0, 0, 0, 1, 0x67, 1, 2, 0, 0, 1, 0x68, 3]));
        assert!(has_sps(&[0, 0, 1, 0x27, 9]));
        assert!(!has_sps(&[0, 0, 0, 1, 0x65, 1, 2, 0, 0, 1, 0x41, 3]));
        assert!(!has_sps(&[]));
    }

    #[test]
    fn pair() {
        assert_eq!(unpack_pair(pack_pair(1280, 720)), (1280, 720));
    }
}

#[cfg(test)]
mod out_sample_tests {
    use super::*;

    #[test]
    fn sample_survives_need_more_input() {
        startup();
        let sample = unsafe { MFCreateSample().unwrap() };
        let mut spare = OutSample::default();
        // simula: take() no início do process_output, devolução em qualquer resultado
        spare.sample = Some(sample.clone());
        let taken = spare.sample.take();
        assert!(spare.sample.is_none());
        spare.give_back(&taken); // caminho NeedMoreInput: sample devolvido pelo MFT
        let again = spare.sample.take().expect("cache perdido");
        assert!(again == sample, "deve ser o mesmo objeto COM");
    }
}
