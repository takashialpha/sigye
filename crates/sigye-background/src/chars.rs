//! Character constants for background animations.

/// Characters used for starfield background.
pub const STAR_CHARS: &[char] = &['.', '*', '+', '·', '✦', '✧'];

/// Characters used for matrix rain.
pub const MATRIX_CHARS: &[char] = &[
    'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ', 'タ',
    'チ', 'ツ', 'テ', 'ト', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

/// Characters used for snowfall background.
pub const SNOW_CHARS: &[char] = &['*', '·', '•', '❄', '❅', '❆', '✦', '✧', '°'];

/// Characters used for frost crystals.
pub const FROST_CHARS: &[char] = &['·', '•', '*', '×', '✕', '✱', '░'];

// Weather character constants

/// Characters used for rain drops - vertical streaks.
pub const RAIN_CHARS: &[char] = &['│', '|', '¦', '┃', '╏', '┊', '┆'];

/// Characters used for heavy storm rain - more intense.
pub const STORM_RAIN_CHARS: &[char] = &['┃', '║', '│', '|', '/', '\\'];

/// Characters used for sun rays and shimmer.
pub const SUN_CHARS: &[char] = &['·', '•', '*', '✦', '✧', '○', '◌'];

/// Characters used for wind streaks - horizontal motion.
pub const WIND_CHARS: &[char] = &['─', '-', '~', '∼', '≈', '━', '╌', '╍'];

/// Characters used for cloud puffs and density.
pub const CLOUD_CHARS: &[char] = &['░', '▒', '▓', '·', '•', '○', '◌', '◦'];

/// Characters used for fog/mist - soft wisps and dots.
pub const FOG_CHARS: &[char] = &['·', '.', '\'', ':', '°', '∙', ','];

/// Characters used for cherry blossom petals.
pub const PETAL_CHARS: &[char] = &['✿', '❀', '✾', '·', '°'];
