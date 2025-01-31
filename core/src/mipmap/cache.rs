//! MIPMap Cache for ImageTexture

use super::convert_in::*;
use super::tex_info::*;
use crate::image_io::*;
use crate::mipmap::*;
use crate::spectrum::*;
use std::collections::HashMap;
use std::marker::{Send, Sync};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign};
use std::result::Result;
use std::sync::{Arc, Mutex};

/// Interface for caching and retrieving `MIPMap`s.
pub trait MIPMapCacheProvider<Tmemory> {
    /// Get the mipmap for a given `TexInfo` from cache; if it doesn't exist load it from file, store it in cache and
    /// return a reference.
    ///
    /// * `tex_info` - Texture information.
    fn get(info: TexInfo) -> MIPMapCacheResult<Tmemory>;
}

/// Type for result of retrieving `MIPMapCacheProvider<Tmemory>::get()`.
pub type MIPMapCacheResult<Tmemory> = Result<ArcMIPMap<Tmemory>, String>;

/// Type for storing `MIPMap`s of type `Tmemory` in a `LazyLock`.
type MIPMaps<Tmemory> = Mutex<HashMap<TexInfo, Arc<MIPMap<Tmemory>>>>;

/// Provides a way to cache and retrieve `MIPMap`s for `ImageTexture`s.
pub struct MIPMapCache {}

macro_rules! cache_provider {
    ($t: ty, $id: ident) => {
        /// Caches `MIPMap<$t>`.
        static $id: std::sync::LazyLock<MIPMaps<$t>> = std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

        impl MIPMapCacheProvider<$t> for MIPMapCache {
            /// Get the mipmap for a given `TexInfo` from cache; if it doesn't exist load it from file, store it in
            /// cache and return a reference.
            ///
            /// * `tex_info` - Texture information.
            fn get(info: TexInfo) -> Result<ArcMIPMap<$t>, String> {
                let mut mipmaps = $id.lock().expect("Unable to access mipmap mutex");
                match mipmaps.get(&info) {
                    Some(mipmap) => Ok(Arc::clone(&mipmap)),
                    None => {
                        let mipmap = generate_mipmap(&info)?;
                        mipmaps.insert(info, Arc::clone(&mipmap));
                        Ok(mipmap)
                    }
                }
            }
        }
    };
}

cache_provider!(RGBSpectrum, RGB_SPECTRUM_MIPMAPS);
cache_provider!(Float, FLOAT_MIPMAPS);

/// Clears all MIPMap caches.
pub fn clear_mipmap_caches() {
    let mut mipmaps = RGB_SPECTRUM_MIPMAPS
        .lock()
        .expect("Unable to access RGB_SPECTRUM_MIPMAPS mutex");
    mipmaps.clear();

    let mut mipmaps = FLOAT_MIPMAPS.lock().expect("Unable to access FLOAT_MIPMAPS mutex");
    mipmaps.clear();
}

/// Load an image texture from file and build the `MIPMap`.
///
/// * `info` - Texture information.
fn generate_mipmap<Tmemory>(info: &TexInfo) -> Result<Arc<MIPMap<Tmemory>>, String>
where
    Tmemory: Copy
        + Default
        + Mul<Float, Output = Tmemory>
        + MulAssign<Float>
        + Div<Float, Output = Tmemory>
        + DivAssign<Float>
        + Add<Tmemory, Output = Tmemory>
        + AddAssign
        + Clamp<Float>
        + Send
        + Sync,
    Spectrum: ConvertIn<Tmemory>,
{
    // Create `MipMap` for `filename`.
    let RGBImage {
        pixels: mut texels,
        resolution,
    } = match read_image(info.path.as_str()) {
        Ok(img) => img,
        Err(err) => return Err(format!("Error reading texture {}, {:}.", info.path, err)),
    };

    // Flip image in y; texture coordinate space has (0,0) at the lower left corner.
    for y in 0..resolution.y / 2 {
        for x in 0..resolution.x {
            let o1 = y * resolution.x + x;
            let o2 = (resolution.y - 1 - y) * resolution.x + x;
            texels.swap(o1, o2);
        }
    }

    // Convert texels to type M and create MIPMap.
    let converted_texels: Vec<Tmemory> = texels
        .iter()
        .map(|texel| texel.convert_in(info.scale, info.gamma))
        .collect();

    Ok(Arc::new(MIPMap::new(
        &resolution,
        &converted_texels,
        info.filtering_method,
        info.wrap_mode,
        info.max_anisotropy,
    )))
}
