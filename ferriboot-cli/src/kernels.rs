use std::{
    cmp::Ordering,
    iter::Peekable,
    path::{Path, PathBuf},
    str::Chars,
};
#[derive(Clone)]
pub struct Kernel {
    pub variant: String,
    pub image: PathBuf,
    pub initrd: Option<PathBuf>,
    pub fallback: Option<PathBuf>,
}

const INITRD_PATTERNS: [(&str, &str); 2] = [
    ("initramfs-", ".img"), // Arch (mkinitcpio), Fedora (dracut)
    ("initrd.img-", ""),    // Ubuntu, Debian
];

fn find_initrd(dir: &Path, variant: &str) -> Option<PathBuf> {
    for (prefix, suffix) in INITRD_PATTERNS {
        let file_name = format!("{}{}{}", prefix, variant, suffix);
        if dir.join(&file_name).exists() {
            return Some(dir.join(&file_name));
        }
    }

    None
}

pub fn find_all_kernels(esp: &Path) -> Result<Vec<Kernel>, String> {
    let mut dirs = Vec::new();
    dirs.push(esp.to_path_buf());

    if let (Ok(boot), Ok(esp_real)) = (std::fs::canonicalize("/boot"), std::fs::canonicalize(esp))
        && boot != esp_real
    {
        dirs.push(boot);
    }

    let mut all = Vec::<Kernel>::new();

    for (i, dir) in dirs.iter().enumerate() {
        match find_kernels(dir) {
            Ok(found) => {
                for kernel in found {
                    if all.iter().any(|k| k.variant == kernel.variant) {
                        continue;
                    }
                    all.push(kernel);
                }
            }
            Err(e) => {
                if i == 0 {
                    return Err(format!("cannot read {}: {}", dir.display(), e));
                }
                eprintln!("cannot read {}: {}", dir.display(), e);
            }
        }
    }
    all.sort_by(|a, b| kernel_cmp(&a.variant, &b.variant));
    all.sort_by_key(|kernel| kernel.variant.starts_with("0-rescue"));
    Ok(all)
}

pub fn find_kernels(esp: &Path) -> std::io::Result<Vec<Kernel>> {
    let mut kernels = Vec::new();
    let result = std::fs::read_dir(esp)?;

    for entry in result {
        if entry.is_err() {
            continue;
        }
        let dir_entry = entry.unwrap();
        let file_name_os = dir_entry.file_name();
        let file_name = file_name_os.to_str();
        if let Some(converted) = file_name {
            if let Some(variant) = converted.strip_prefix("vmlinuz-") {
                let initrd = find_initrd(esp, variant);
                let fallback_path = format!("initramfs-{}-fallback.img", variant);
                let fallback = esp.join(&fallback_path);
                let fallback = fallback.exists().then_some(fallback);
                kernels.push(Kernel {
                    variant: variant.to_string(),
                    image: esp.join(converted),
                    initrd,
                    fallback,
                });
            } else {
                continue;
            }
        } else {
            continue;
        }
    }

    Ok(kernels)
}
#[allow(clippy::question_mark)]
fn next_chunk(it: &mut Peekable<Chars>) -> Option<String> {
    let Some(next) = it.peek() else {
        return None;
    };

    let is_digit = next.is_ascii_digit();
    let mut str = String::new();

    while let Some(next) = it.peek() {
        if next.is_ascii_digit() != is_digit {
            break;
        }
        str.push(it.next().unwrap());
    }

    Some(str)
}

fn natural_cmp(a: &str, b: &str) -> Ordering {
    let mut it_a = a.chars().peekable();
    let mut it_b = b.chars().peekable();

    loop {
        match (next_chunk(&mut it_a), next_chunk(&mut it_b)) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let both_digits = x.chars().next().is_some_and(|c| c.is_ascii_digit())
                    && y.chars().next().is_some_and(|c| c.is_ascii_digit());
                let ord = if both_digits {
                    let xt = x.trim_start_matches('0');
                    let yt = y.trim_start_matches('0');
                    xt.len().cmp(&yt.len()).then_with(|| xt.cmp(yt))
                } else {
                    x.cmp(&y)
                };

                if ord != Ordering::Equal {
                    return ord;
                }
            }
        }
    }
}
pub(crate) fn kernel_cmp(a: &str, b: &str) -> Ordering {
    let both_digits = a.chars().next().is_some_and(|c| c.is_ascii_digit())
        && b.chars().next().is_some_and(|c| c.is_ascii_digit());
    if both_digits {
        natural_cmp(b, a)
    } else {
        natural_cmp(a, b)
    }
}
