use std::process;

pub const USAGE: &str = "\
İstifadə: asciify <şəkil> [seçimlər]

  -o, --output <fayl>   çıxış PNG faylı (default: ascii.png)
      --crop x1,y1,x2,y2 yalnız bu düzbucaqlını götür (piksellə)
      --cols <n>        simvol sayı (en), default 100
      --gamma <f>       >1 qaraldır, <1 işıqlandırır, default 1.6
      --size <px>       çıxış şriftinin ölçüsü, default 14 (böyük = daha yüksək PNG)
      --soft-bg         fonu bulanıqlaşdırıb yumşaq qaralt (alpha yoxdursa)
      --light           ağ fon + qara simvollar (default: qaranlıq fon)
  -h, --help            bu mətn";

const DEFAULT_OUTPUT: &str = "ascii.png";
const DEFAULT_COLS: u32 = 100;
const DEFAULT_GAMMA: f32 = 1.6;
const DEFAULT_FONT_PX: f32 = 14.0;
const MIN_COLS: u32 = 10;

pub struct Opts {
    pub input: String,
    pub output: String,
    pub crop: Option<(u32, u32, u32, u32)>,
    pub cols: u32,
    pub gamma: f32,
    pub font_px: f32,
    pub soft_bg: bool,
    pub light: bool,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            input: String::new(),
            output: DEFAULT_OUTPUT.into(),
            crop: None,
            cols: DEFAULT_COLS,
            gamma: DEFAULT_GAMMA,
            font_px: DEFAULT_FONT_PX,
            soft_bg: false,
            light: false,
        }
    }
}

pub fn parse_args() -> Result<Opts, String> {
    let mut opts = Opts::default();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        let mut value = |name: &str| {
            args.next().ok_or(format!("{name} üçün dəyər yoxdur"))
        };

        match arg.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                process::exit(0);
            }
            "-o" | "--output" => opts.output = value("--output")?,
            "--crop" => opts.crop = Some(parse_crop(&value("--crop")?)?),
            "--cols" | "--width" => {
                opts.cols = value("--cols")?
                    .parse()
                    .map_err(|e| format!("--cols: {e}"))?
            }
            "--gamma" => {
                opts.gamma = value("--gamma")?
                    .parse()
                    .map_err(|e| format!("--gamma: {e}"))?
            }
            "--size" => {
                opts.font_px = value("--size")?
                    .parse()
                    .map_err(|e| format!("--size: {e}"))?
            }
            "--soft-bg" => opts.soft_bg = true,
            "--light" => opts.light = true,
            unknown if unknown.starts_with('-') => {
                return Err(format!("Naməlum seçim: {unknown}\n\n{USAGE}"));
            }
            input => opts.input = input.to_string(),
        }
    }

    if opts.input.is_empty() {
        return Err(format!("Şəkil faylı verilməyib.\n\n{USAGE}"));
    }

    if opts.cols < MIN_COLS {
        return Err(format!("--cols ən azı {MIN_COLS} olmalıdır"));
    }

    // A NaN gamma turns every pixel into NaN and yields an empty image, so report an
    // error rather than a silently broken result.
    if opts.gamma.is_nan() {
        return Err("--gamma ədəd olmalıdır".into());
    }

    Ok(opts)
}

/// `x1,y1,x2,y2` — top-left and bottom-right corner coordinates.
fn parse_crop(raw: &str) -> Result<(u32, u32, u32, u32), String> {
    let coords: Vec<u32> = raw
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<u32>()
                .map_err(|e| format!("--crop: {e}"))
        })
        .collect::<Result<_, _>>()?;

    let [x1, y1, x2, y2] = match coords[..] {
        [x1, y1, x2, y2] => [x1, y1, x2, y2],
        _ => return Err("--crop formatı: x1,y1,x2,y2 (x2>x1, y2>y1)".into()),
    };

    if x2 <= x1 || y2 <= y1 {
        return Err("--crop formatı: x1,y1,x2,y2 (x2>x1, y2>y1)".into());
    }

    Ok((x1, y1, x2, y2))
}
