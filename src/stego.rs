use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView};

/// Marqueur magique pour identifier un coffre-fort pixelvault
const MAGIC: &[u8] = b"PXVLT";

/// Encode des données dans une image en utilisant LSB (Least Significant Bit)
/// Principe: modifie le bit de poids faible des canaux RGB
pub fn encode(image: &DynamicImage, data: &[u8]) -> Result<DynamicImage> {
    let (width, height) = image.dimensions();
    let mut img_buffer = image.to_rgba8();

    // Calcule la capacité disponible (3 bits par pixel pour RGB)
    let capacity = (width * height * 3) / 8;

    // Format: [MAGIC (5 bytes) || longueur (4 bytes) || données]
    let total_len = MAGIC.len() + 4 + data.len();

    if total_len > capacity as usize {
        anyhow::bail!(
            "Image trop petite: capacité {} bytes, besoin de {} bytes",
            capacity,
            total_len
        );
    }

    // Prépare le payload
    let mut payload = Vec::new();
    payload.extend_from_slice(MAGIC);
    payload.extend_from_slice(&(data.len() as u32).to_be_bytes());
    payload.extend_from_slice(data);

    // Convertit en bits
    let bits: Vec<bool> = payload
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1 == 1))
        .collect();

    // Encode les bits dans l'image
    let mut bit_index = 0;
    'outer: for y in 0..height {
        for x in 0..width {
            let pixel = img_buffer.get_pixel_mut(x, y);

            // Encode dans R, G, B (pas A pour préserver la transparence)
            for channel in 0..3 {
                if bit_index >= bits.len() {
                    break 'outer;
                }

                let bit = bits[bit_index];
                pixel[channel] = (pixel[channel] & 0xFE) | (bit as u8);
                bit_index += 1;
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(img_buffer))
}

/// Décode des données depuis une image
pub fn decode(image: &DynamicImage) -> Result<Vec<u8>> {
    let (width, height) = image.dimensions();
    let img_buffer = image.to_rgba8();

    // Extrait les bits
    let mut bits = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let pixel = img_buffer.get_pixel(x, y);

            // Lit les 3 canaux RGB
            for channel in 0..3 {
                bits.push((pixel[channel] & 1) == 1);
            }
        }
    }

    // Convertit les bits en bytes
    let bytes: Vec<u8> = bits
        .chunks(8)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .fold(0u8, |acc, (i, &bit)| acc | ((bit as u8) << (7 - i)))
        })
        .collect();

    // Vérifie le marqueur magique
    if bytes.len() < MAGIC.len() {
        anyhow::bail!("Aucune donnée trouvée dans l'image");
    }

    if &bytes[..MAGIC.len()] != MAGIC {
        anyhow::bail!("Pas de coffre-fort pixelvault dans cette image");
    }

    // Lit la longueur
    if bytes.len() < MAGIC.len() + 4 {
        anyhow::bail!("Données corrompues (longueur manquante)");
    }

    let len_bytes: [u8; 4] = bytes[MAGIC.len()..MAGIC.len() + 4]
        .try_into()
        .context("Erreur lecture longueur")?;
    let data_len = u32::from_be_bytes(len_bytes) as usize;

    // Extrait les données
    let data_start = MAGIC.len() + 4;
    let data_end = data_start + data_len;

    if data_end > bytes.len() {
        anyhow::bail!("Données corrompues (longueur invalide)");
    }

    Ok(bytes[data_start..data_end].to_vec())
}

/// Calcule la capacité de stockage d'une image en bytes
#[allow(dead_code)]
pub fn calculate_capacity(width: u32, height: u32) -> usize {
    let total_bits = width as usize * height as usize * 3; // 3 canaux RGB
    let header_size = MAGIC.len() + 4; // Magic + longueur

    (total_bits / 8).saturating_sub(header_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn test_encode_decode() {
        // Crée une image de test
        let img =
            DynamicImage::ImageRgba8(ImageBuffer::from_pixel(100, 100, Rgba([255, 0, 0, 255])));

        let data = b"Test data for steganography";

        let encoded = encode(&img, data).unwrap();
        let decoded = decode(&encoded).unwrap();

        assert_eq!(data.to_vec(), decoded);
    }
}
