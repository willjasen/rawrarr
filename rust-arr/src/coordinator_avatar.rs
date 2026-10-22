//! Deterministic coordinator profile avatar rendering and JPEG XL encoding.
use std::{
    fs,
    process::{Command, Stdio},
};
use uuid::Uuid;

fn svg_for_digest(digest: &[u8; 32]) -> String {
    let mut index = 0usize;
    let mut random = || {
        let value = digest[index % digest.len()] as f32 / 256.0;
        index += 1;
        value
    };
    let golden = digest[31] < 5;
    let hue = (random() * 360.0).floor() as u16;
    let body = if golden {
        "hsl(44 82% 58%)".to_owned()
    } else {
        format!("hsl({hue} 58% 57%)")
    };
    let light = if golden {
        "hsl(48 92% 78%)".to_owned()
    } else {
        format!("hsl({} 78% 78%)", (hue + 42) % 360)
    };
    let dark = if golden {
        "hsl(31 70% 30%)".to_owned()
    } else {
        format!("hsl({} 52% 32%)", (hue + 350) % 360)
    };
    let background = format!("hsl({} 34% 18%)", (hue + 165) % 360);
    let species = (random() * 3.0).floor() as usize;
    let tail_tip = 75 - (20 + (random() * 35.0).floor() as i32);
    let tail_mid = (75 + tail_tip + 1) / 2;
    let tail = if species == 1 {
        format!(
            "M75 112Q{tail_mid} 99 {tail_tip} 109L{} 97Q{} 108 71 127Z",
            tail_tip + 8,
            tail_mid + 5
        )
    } else {
        format!(
            "M75 112Q{tail_mid} 95 {tail_tip} 103Q{} 117 71 127Z",
            tail_tip + 7
        )
    };
    let wings = if random() < 0.34 {
        format!(
            "<path class=\"wing\" d=\"M91 116Q72 111 61 94Q58 87 60 78Q72 81 82 91Q77 80 78 71Q94 82 102 101Q105 109 99 116Z\" fill=\"{light}\" stroke=\"{dark}\" stroke-width=\"3\" stroke-linejoin=\"round\"/><path d=\"M91 109Q78 101 68 88m25 15Q86 91 82 81\" fill=\"none\" stroke=\"{dark}\" stroke-width=\"2\" stroke-linecap=\"round\" opacity=\".7\"/>"
        )
    } else {
        String::new()
    };
    let scene = match digest[30] % 6 {
        0 => {
            "<path d=\"M0 140 35 82l25 33 27-48 33 55 24-36 48 60v46H0Z\" fill=\"#324752\"/><path d=\"m56 103 31-36 20 35-14-6-8 11-10-8-8 9z\" fill=\"#e77b52\"/><path d=\"M0 157q38-18 75 0t75 0q24-9 42 0v35H0Z\" fill=\"#263d37\"/><path d=\"M25 151v-31m-13 12 13-15 12 15m43 20v-26m-11 11 11-14 11 14\" fill=\"none\" stroke=\"#678c68\" stroke-width=\"7\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>"
        }
        1 => {
            "<path d=\"M0 142q48-19 96 0t96 0v50H0Z\" fill=\"#31453a\"/><path d=\"M0 161q45-14 91 0t101 0v31H0Z\" fill=\"#263d37\"/><path d=\"M12 140V79m-11 17Q-5 69 12 55q21 12 18 41M0 78q2-19 12-23 16 5 19 23m145 64V70m-15 23q-6-30 15-43 22 13 17 43m-32-17q1-20 15-25 14 5 17 25\" fill=\"none\" stroke=\"#a5ce82\" stroke-width=\"9\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M12 91v49m132-59v59\" stroke=\"#d3a06b\" stroke-width=\"8\" stroke-linecap=\"round\"/><circle cx=\"9\" cy=\"58\" r=\"2.5\" fill=\"#e3f1b2\"/><circle cx=\"178\" cy=\"51\" r=\"2.5\" fill=\"#e3f1b2\"/>"
        }
        2 => {
            "<path d=\"M0 150 31 125l22 15 30-35 26 29 26-18 57 37v39H0Z\" fill=\"#384b46\"/><path d=\"M0 169q42-20 84 0t108 0v23H0Z\" fill=\"#263d37\"/><circle cx=\"37\" cy=\"51\" r=\"13\" fill=\"#f2be6a\" opacity=\".78\"/><path d=\"M32 148v-28m-10 10 10-13 10 13m104 19v-25m-10 10 10-13 10 13\" fill=\"none\" stroke=\"#64845c\" stroke-width=\"6\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>"
        }
        3 => {
            "<path d=\"M0 150 28 130l31-43 31 43 19-19 35 39 20-27 28 24v45H0Z\" fill=\"#7b5947\"/><path d=\"m44 108 15-21 16 22-14-7-8 7zM121 129l14-18 14 18-13-6-7 6z\" fill=\"#d6a879\"/><path d=\"M0 166q39-14 78 0t114 0v26H0Z\" fill=\"#bd8954\"/><path d=\"M33 156v-16m0 7-10-8m10 12 10-9m113 15v-22m0 9-10-9m10 12 11-10m-11 15v-9m-6 0h12\" fill=\"none\" stroke=\"#64845c\" stroke-width=\"5\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>"
        }
        4 => {
            "<path d=\"M0 147 39 90l27 36 31-58 36 63 24-34 35 42v53H0Z\" fill=\"#829ca0\"/><path d=\"m76 86 21-18 20 35-16-8-8 10-8-9-9 5z\" fill=\"#eef5ed\"/><path d=\"M0 163q42-17 84 0t108 0v29H0Z\" fill=\"#51685e\"/><path d=\"M28 151v-23m-9 7 9-12 9 12m124 15v-28m-11 9 11-14 11 14\" fill=\"none\" stroke=\"#66866d\" stroke-width=\"6\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><circle cx=\"31\" cy=\"123\" r=\"3\" fill=\"#e9f3e8\"/><circle cx=\"153\" cy=\"119\" r=\"3\" fill=\"#e9f3e8\"/>"
        }
        _ => {
            "<path d=\"M0 148q38-18 78 0t114 0v44H0Z\" fill=\"#497d76\"/><path d=\"M0 168q43-15 86 0t106 0v24H0Z\" fill=\"#32635c\"/><circle cx=\"35\" cy=\"54\" r=\"16\" fill=\"#f4cb78\" opacity=\".9\"/><path d=\"M30 151q8-33 0-64m0 10Q15 92 9 77q17 0 26 14m-5-9q3-17 18-25 1 17-12 29m-3-12q-11-12-9-28 15 7 17 25\" fill=\"none\" stroke=\"#8dbb71\" stroke-width=\"6\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M124 156v-21m-8 8 8-10 8 10\" fill=\"none\" stroke=\"#b8d98d\" stroke-width=\"5\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>"
        }
    };
    let spike_count = if wings.is_empty() {
        (random() * 4.0).floor() as usize
    } else {
        0
    };
    let positions: Vec<i32> = match spike_count {
        1 => vec![96 + (random() * 9.0).floor() as i32],
        2 => {
            if random() < 0.5 {
                vec![82, 108]
            } else {
                vec![88, 114]
            }
        }
        3 => vec![76, 94, 112],
        _ => Vec::new(),
    };
    let heights: Vec<i32> = match spike_count {
        1 => vec![22 + (random() * 10.0).floor() as i32],
        2 => {
            let tall = 19 + (random() * 9.0).floor() as i32;
            let mut short = 12 + (random() * 9.0).floor() as i32;
            if short == tall {
                short += if tall > 20 { -1 } else { 1 };
            }
            vec![tall, short]
        }
        3 => positions
            .iter()
            .map(|_| 13 + (random() * 13.0).floor() as i32)
            .collect(),
        _ => Vec::new(),
    };
    let spikes = positions.iter().zip(heights.iter()).map(|(x, height)| {
        let width = match spike_count { 1 => 10, 2 => 9, _ => 7 };
        format!("<path d=\"M{x} 97l{} -{height} {} {height}z\" fill=\"{light}\" stroke=\"{dark}\" stroke-width=\"2\" stroke-linejoin=\"round\"/>", width as f32 / 2.0, width as f32 / 2.0)
    }).collect::<String>();
    let spots = (0..(2 + (random() * 4.0).floor() as usize))
        .map(|_| {
            let x = 83 + (random() * 42.0).floor() as i32;
            let y = 111 + (random() * 22.0).floor() as i32;
            let r = 2 + (random() * 3.0).floor() as i32;
            format!("<circle cx=\"{x}\" cy=\"{y}\" r=\"{r}\" fill=\"{light}\" opacity=\".8\"/>")
        })
        .collect::<String>();
    let crest = match species {
        0 => format!(
            "<path d=\"M133 80q11 -17 18 -1q-8 8 -18 7z\" fill=\"{light}\" stroke=\"{dark}\" stroke-width=\"2\"/>"
        ),
        1 => format!(
            "<path d=\"M136 83l7 -13 5 13m4 0 8 -11 3 14\" fill=\"{light}\" stroke=\"{dark}\" stroke-width=\"3\" stroke-linejoin=\"round\"/>"
        ),
        _ => format!(
            "<path d=\"M137 83q3 -17 12 -16q10 2 10 18\" fill=\"none\" stroke=\"{light}\" stroke-width=\"5\" stroke-linecap=\"round\"/>"
        ),
    };
    let sparkles = if golden {
        "<path d=\"M49 84l2 5 5 2-5 2-2 5-2-5-5-2 5-2zM153 49l1.5 4 4 1.5-4 1.5-1.5 4-1.5-4-4-1.5 4-1.5z\" fill=\"#fff1b8\" opacity=\".9\"/>"
    } else {
        ""
    };
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 192 192\"><rect width=\"192\" height=\"192\" rx=\"96\" fill=\"{background}\"/><circle cx=\"39\" cy=\"48\" r=\"18\" fill=\"{light}\" opacity=\".12\"/><circle cx=\"154\" cy=\"145\" r=\"27\" fill=\"{light}\" opacity=\".09\"/>{scene}{sparkles}<path class=\"tail\" d=\"{tail}\" fill=\"{body}\" stroke=\"{dark}\" stroke-width=\"4\" stroke-linejoin=\"round\"/><ellipse cx=\"96\" cy=\"119\" rx=\"39\" ry=\"29\" fill=\"{body}\" stroke=\"{dark}\" stroke-width=\"4\"/>{wings}<path d=\"M83 133v16m29 -16v16\" stroke=\"{dark}\" stroke-width=\"9\" stroke-linecap=\"round\"/><path d=\"M84 143h-8m8 4h-9m37 -4h9m-9 4h10\" stroke=\"{light}\" stroke-width=\"3\" stroke-linecap=\"round\"/><path d=\"M111 107Q121 93 132 88Q139 85 148 89L159 98Q163 102 158 107L148 112Q139 120 126 117Z\" fill=\"{body}\" stroke=\"{dark}\" stroke-width=\"4\" stroke-linejoin=\"round\"/>{crest}{spikes}<ellipse cx=\"151\" cy=\"102\" rx=\"2.6\" ry=\"3.2\" fill=\"#18201b\"/><circle cx=\"152\" cy=\"101\" r=\".9\" fill=\"#fff\"/><path d=\"M154 110q-7 6 -14 2\" fill=\"none\" stroke=\"{dark}\" stroke-width=\"2.5\" stroke-linecap=\"round\"/><path d=\"M125 117q5 2 10 1\" fill=\"none\" stroke=\"{light}\" stroke-width=\"3\" stroke-linecap=\"round\"/>{spots}<path d=\"M78 88q-8 -8 -15 -3\" fill=\"none\" stroke=\"{light}\" stroke-width=\"3\" stroke-linecap=\"round\" opacity=\".8\"/></svg>"
    )
}

fn fossil_svg_for_digest(digest: &[u8; 32]) -> String {
    let hue = (digest[0] as u16 * 360 / 256 + 24) % 360;
    let rock = format!("hsl({hue} 24% 31%)");
    let rock_light = format!("hsl({hue} 26% 43%)");
    let fossil = format!("hsl({} 48% 73%)", (hue + 34) % 360);
    let fossil_shadow = format!("hsl({} 35% 48%)", (hue + 34) % 360);
    let background = format!("hsl({} 35% 15%)", (hue + 175) % 360);
    let ribs = (0..12)
        .map(|index| {
            let angle = index * 30 + (digest[1] as usize % 10);
            let control_y = 102 - (digest[2] % 7) as i32;
            let end_y = 102 - (digest[3] % 8) as i32;
            format!(
                "<path d=\"M96 102Q124 {control_y} 131 {end_y}\" transform=\"rotate({angle} 96 102)\" fill=\"none\" stroke=\"{fossil_shadow}\" stroke-width=\"2.5\" stroke-linecap=\"round\"/>"
            )
        })
        .collect::<String>();
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 192 192\"><rect width=\"192\" height=\"192\" rx=\"96\" fill=\"{background}\"/><circle cx=\"39\" cy=\"46\" r=\"22\" fill=\"{fossil}\" opacity=\".1\"/><path d=\"M20 139 49 72l32 14 31-29 58 38 8 53-34 20-68-4-47 8z\" fill=\"{rock}\" stroke=\"{rock_light}\" stroke-width=\"4\" stroke-linejoin=\"round\"/><path d=\"m25 140 29-61 25 12m39-30 48 32 8 45m-113 20 53-7 48 10\" fill=\"none\" stroke=\"{rock_light}\" stroke-width=\"2\" opacity=\".8\"/><circle cx=\"96\" cy=\"102\" r=\"43\" fill=\"{fossil}\" stroke=\"{fossil_shadow}\" stroke-width=\"4\"/>{ribs}<path d=\"M96 102c-10-3-19 4-17 14 2 11 16 15 26 9 13-8 13-28 0-37-17-12-41-2-45 19-5 27 20 47 46 39 29-8 39-43 23-67-18-27-57-31-81-10\" fill=\"none\" stroke=\"{fossil_shadow}\" stroke-width=\"5\" stroke-linecap=\"round\"/><circle cx=\"96\" cy=\"102\" r=\"4\" fill=\"{fossil_shadow}\"/><path d=\"m47 151 10-4 7 5m66-83 8-5 7 7\" fill=\"none\" stroke=\"{fossil}\" stroke-width=\"3\" stroke-linecap=\"round\" opacity=\".85\"/></svg>"
    )
}

fn parse_seed(seed: &str) -> Option<[u8; 32]> {
    if seed.len() != 64 || !seed.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut digest = [0u8; 32];
    for (index, chunk) in seed.as_bytes().chunks_exact(2).enumerate() {
        digest[index] = u8::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?;
    }
    Some(digest)
}

pub fn valid_seed(seed: &str) -> bool {
    parse_seed(seed).is_some()
}

fn rasterize_with(
    seed: &str,
    renderer: fn(&[u8; 32]) -> String,
) -> Result<(std::path::PathBuf, DirectoryCleanup), ()> {
    let digest = parse_seed(seed).ok_or(())?;
    let svg = renderer(&digest);
    let directory = std::env::temp_dir().join(format!("rawrarr-avatar-{}", Uuid::new_v4()));
    fs::create_dir(&directory).map_err(|_| ())?;
    let _cleanup = DirectoryCleanup(directory.clone());
    let svg_path = directory.join("avatar.svg");
    let png_path = directory.join("avatar.png");
    fs::write(&svg_path, svg).map_err(|_| ())?;
    let raster = Command::new("rsvg-convert")
        .arg("--format=png")
        .arg("--output")
        .arg(&png_path)
        .arg(&svg_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| ())?;
    if !raster.success() {
        return Err(());
    }
    Ok((png_path, _cleanup))
}

fn rasterize(seed: &str) -> Result<(std::path::PathBuf, DirectoryCleanup), ()> {
    rasterize_with(seed, svg_for_digest)
}

fn rasterize_fossil(seed: &str) -> Result<(std::path::PathBuf, DirectoryCleanup), ()> {
    rasterize_with(seed, fossil_svg_for_digest)
}

pub fn render_png(seed: &str) -> Result<Vec<u8>, ()> {
    let (png_path, _cleanup) = rasterize(seed)?;
    fs::read(png_path).map_err(|_| ())
}

pub fn render_jxl(seed: &str) -> Result<Vec<u8>, ()> {
    let (png_path, _cleanup) = rasterize(seed)?;
    let jxl_path = png_path.parent().ok_or(())?.join("avatar.jxl");
    let encoded = Command::new("cjxl")
        .arg(&png_path)
        .arg(&jxl_path)
        .arg("-d")
        .arg("0")
        .arg("-e")
        .arg("7")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| ())?;
    if !encoded.success() {
        return Err(());
    }
    fs::read(jxl_path).map_err(|_| ())
}

pub fn render_fossil_png(seed: &str) -> Result<Vec<u8>, ()> {
    let (png_path, _cleanup) = rasterize_fossil(seed)?;
    fs::read(png_path).map_err(|_| ())
}

pub fn render_fossil_jxl(seed: &str) -> Result<Vec<u8>, ()> {
    let (png_path, _cleanup) = rasterize_fossil(seed)?;
    let jxl_path = png_path.parent().ok_or(())?.join("avatar.jxl");
    let encoded = Command::new("cjxl")
        .arg(&png_path)
        .arg(&jxl_path)
        .arg("-d")
        .arg("0")
        .arg("-e")
        .arg("7")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| ())?;
    if !encoded.success() {
        return Err(());
    }
    fs::read(jxl_path).map_err(|_| ())
}

struct DirectoryCleanup(std::path::PathBuf);
impl Drop for DirectoryCleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::{fossil_svg_for_digest, parse_seed, svg_for_digest};
    use sha2::{Digest, Sha256};
    use std::collections::BTreeSet;

    #[test]
    fn validates_avatar_seeds_as_exact_sha256_hex() {
        assert!(parse_seed(&"ab".repeat(32)).is_some());
        assert!(parse_seed("abcd").is_none());
        assert!(parse_seed(&"zz".repeat(32)).is_none());
    }

    #[test]
    fn fossil_device_art_is_deterministic_and_uses_a_shell_fossil() {
        let digest: [u8; 32] = Sha256::digest("device-fossil").into();
        let fossil = fossil_svg_for_digest(&digest);
        assert_eq!(fossil, fossil_svg_for_digest(&digest));
        assert!(fossil.contains("<circle cx=\"96\" cy=\"102\" r=\"43\""));
        assert!(fossil.contains("stroke-linecap=\"round\""));
        assert!(!fossil.contains("<ellipse cx=\"96\" cy=\"119\""));
        assert_ne!(fossil, svg_for_digest(&digest));
    }

    #[test]
    fn avatar_scenes_repeat_and_winged_dinosaurs_have_no_spikes() {
        let mut scenes = BTreeSet::new();
        for index in 0..256 {
            let digest: [u8; 32] = Sha256::digest(format!("avatar-scene-{index}")).into();
            let svg = svg_for_digest(&digest);
            assert_eq!(svg, svg_for_digest(&digest));
            scenes.insert(digest[30] % 6);
            let winged = svg.contains("class=\"wing\"");
            let spike_count = svg.matches(" 97l").count();
            if winged {
                assert_eq!(spike_count, 0);
            }
        }
        assert_eq!(scenes, BTreeSet::from([0, 1, 2, 3, 4, 5]));
    }

    #[test]
    fn avatar_traits_vary_and_golden_palette_is_rare() {
        let mut tails = BTreeSet::new();
        let mut spike_counts = BTreeSet::new();
        let mut golden_count = 0usize;
        for index in 0..2048 {
            let digest: [u8; 32] = Sha256::digest(format!("avatar-traits-{index}")).into();
            let svg = svg_for_digest(&digest);
            let tail = svg
                .split("<path class=\"tail\" d=\"")
                .nth(1)
                .unwrap_or_default()
                .split('"')
                .next()
                .unwrap_or_default();
            tails.insert(tail.to_owned());
            let spikes = svg.matches(" 97l").count();
            spike_counts.insert(spikes);
            if svg.contains("hsl(44 82% 58%)") {
                golden_count += 1;
            }
            if svg.contains("class=\"wing\"") {
                assert_eq!(spikes, 0);
            }
        }
        assert!(tails.len() > 3);
        assert_eq!(spike_counts, BTreeSet::from([0, 1, 2, 3]));
        assert!(golden_count > 0 && golden_count < 2048 / 20);
    }

    #[test]
    fn encodes_generated_avatar_as_a_jpeg_xl_codestream() {
        let bytes = super::render_jxl(&"01".repeat(32)).expect("JPEG XL tools are installed");
        assert_eq!(&bytes[..2], &[0xff, 0x0a]);
    }

    #[test]
    fn provides_a_png_raster_fallback() {
        let bytes = super::render_png(&"01".repeat(32)).expect("SVG rasterizer is installed");
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }
}
