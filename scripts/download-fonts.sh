#!/bin/bash
set -e
FONTS_DIR="/Users/sachin/Desktop/melp/rdrive/doc-engine/editor/fonts"
cd "$FONTS_DIR"

# Google Fonts base URL
GF="https://github.com/google/fonts/raw/main"

download() {
  local url="$1"
  local name="$2"
  if [ ! -f "$name" ]; then
    echo "  Downloading $name..."
    curl -sL "$url" -o "$name" 2>/dev/null || echo "  WARN: Failed $name"
  fi
}

echo "=== Downloading metric-compatible MS Office fonts ==="
# Carlito (Calibri replacement) - Croscore
download "$GF/ofl/carlito/Carlito-Regular.ttf" "Carlito-Regular.ttf"
download "$GF/ofl/carlito/Carlito-Bold.ttf" "Carlito-Bold.ttf"
download "$GF/ofl/carlito/Carlito-Italic.ttf" "Carlito-Italic.ttf"
download "$GF/ofl/carlito/Carlito-BoldItalic.ttf" "Carlito-BoldItalic.ttf"

# Caladea (Cambria replacement) - Croscore
download "$GF/ofl/caladea/Caladea-Regular.ttf" "Caladea-Regular.ttf"
download "$GF/ofl/caladea/Caladea-Bold.ttf" "Caladea-Bold.ttf"
download "$GF/ofl/caladea/Caladea-Italic.ttf" "Caladea-Italic.ttf"
download "$GF/ofl/caladea/Caladea-BoldItalic.ttf" "Caladea-BoldItalic.ttf"

# Tinos (Times New Roman replacement) - Croscore
download "$GF/apache/tinos/Tinos-Regular.ttf" "Tinos-Regular.ttf"
download "$GF/apache/tinos/Tinos-Bold.ttf" "Tinos-Bold.ttf"
download "$GF/apache/tinos/Tinos-Italic.ttf" "Tinos-Italic.ttf"
download "$GF/apache/tinos/Tinos-BoldItalic.ttf" "Tinos-BoldItalic.ttf"

# Arimo (Arial replacement) - Croscore
download "$GF/apache/arimo/Arimo%5Bwght%5D.ttf" "Arimo-Regular.ttf"
download "$GF/apache/arimo/Arimo-Italic%5Bwght%5D.ttf" "Arimo-Italic.ttf"

# Cousine (Courier New replacement) - Croscore
download "$GF/apache/cousine/Cousine-Regular.ttf" "Cousine-Regular.ttf"
download "$GF/apache/cousine/Cousine-Bold.ttf" "Cousine-Bold.ttf"
download "$GF/apache/cousine/Cousine-Italic.ttf" "Cousine-Italic.ttf"
download "$GF/apache/cousine/Cousine-BoldItalic.ttf" "Cousine-BoldItalic.ttf"

echo "=== Downloading common document fonts ==="
# Roboto
download "$GF/ofl/roboto/Roboto%5Bwdth%2Cwght%5D.ttf" "Roboto-Regular.ttf"
download "$GF/ofl/roboto/Roboto-Italic%5Bwdth%2Cwght%5D.ttf" "Roboto-Italic.ttf"

# Noto Sans (covers Latin, Cyrillic, Greek)
download "$GF/ofl/notosans/NotoSans%5Bwdth%2Cwght%5D.ttf" "NotoSans-Regular.ttf"
download "$GF/ofl/notosans/NotoSans-Italic%5Bwdth%2Cwght%5D.ttf" "NotoSans-Italic.ttf"

# Noto Serif
download "$GF/ofl/notoserif/NotoSerif%5Bwdth%2Cwght%5D.ttf" "NotoSerif-Regular.ttf"
download "$GF/ofl/notoserif/NotoSerif-Italic%5Bwdth%2Cwght%5D.ttf" "NotoSerif-Italic.ttf"

# Open Sans
download "$GF/ofl/opensans/OpenSans%5Bwdth%2Cwght%5D.ttf" "OpenSans-Regular.ttf"
download "$GF/ofl/opensans/OpenSans-Italic%5Bwdth%2Cwght%5D.ttf" "OpenSans-Italic.ttf"

# Lato
download "$GF/ofl/lato/Lato-Regular.ttf" "Lato-Regular.ttf"
download "$GF/ofl/lato/Lato-Bold.ttf" "Lato-Bold.ttf"
download "$GF/ofl/lato/Lato-Italic.ttf" "Lato-Italic.ttf"
download "$GF/ofl/lato/Lato-BoldItalic.ttf" "Lato-BoldItalic.ttf"

# Source Sans Pro / Source Sans 3
download "$GF/ofl/sourcesans3/SourceSans3%5Bwght%5D.ttf" "SourceSans3-Regular.ttf"
download "$GF/ofl/sourcesans3/SourceSans3-Italic%5Bwght%5D.ttf" "SourceSans3-Italic.ttf"

# Merriweather
download "$GF/ofl/merriweather/Merriweather-Regular.ttf" "Merriweather-Regular.ttf"
download "$GF/ofl/merriweather/Merriweather-Bold.ttf" "Merriweather-Bold.ttf"
download "$GF/ofl/merriweather/Merriweather-Italic.ttf" "Merriweather-Italic.ttf"

# PT Sans / PT Serif
download "$GF/ofl/ptsans/PT_Sans-Web-Regular.ttf" "PTSans-Regular.ttf"
download "$GF/ofl/ptsans/PT_Sans-Web-Bold.ttf" "PTSans-Bold.ttf"
download "$GF/ofl/ptsans/PT_Sans-Web-Italic.ttf" "PTSans-Italic.ttf"
download "$GF/ofl/ptserif/PT_Serif-Web-Regular.ttf" "PTSerif-Regular.ttf"
download "$GF/ofl/ptserif/PT_Serif-Web-Bold.ttf" "PTSerif-Bold.ttf"
download "$GF/ofl/ptserif/PT_Serif-Web-Italic.ttf" "PTSerif-Italic.ttf"

# Georgia-compatible: EB Garamond
download "$GF/ofl/ebgaramond/EBGaramond%5Bwght%5D.ttf" "EBGaramond-Regular.ttf"
download "$GF/ofl/ebgaramond/EBGaramond-Italic%5Bwght%5D.ttf" "EBGaramond-Italic.ttf"

# Liberation fonts (MS Office compatible)
download "https://github.com/liberationfonts/liberation-fonts/files/7261482/liberation-fonts-ttf-2.1.5.tar.gz" "/tmp/liberation.tar.gz"
if [ -f "/tmp/liberation.tar.gz" ]; then
  cd /tmp && tar xzf liberation.tar.gz 2>/dev/null || true
  for f in /tmp/liberation-fonts-ttf-*/Liberation*.ttf; do
    [ -f "$f" ] && cp "$f" "$FONTS_DIR/" 2>/dev/null || true
  done
  cd "$FONTS_DIR"
fi

# Inter (modern UI font)
download "$GF/ofl/inter/Inter%5Bopsz%2Cwght%5D.ttf" "Inter-Regular.ttf"
download "$GF/ofl/inter/Inter-Italic%5Bopsz%2Cwght%5D.ttf" "Inter-Italic.ttf"

# Montserrat
download "$GF/ofl/montserrat/Montserrat%5Bwght%5D.ttf" "Montserrat-Regular.ttf"
download "$GF/ofl/montserrat/Montserrat-Italic%5Bwght%5D.ttf" "Montserrat-Italic.ttf"

# Playfair Display
download "$GF/ofl/playfairdisplay/PlayfairDisplay%5Bwght%5D.ttf" "PlayfairDisplay-Regular.ttf"
download "$GF/ofl/playfairdisplay/PlayfairDisplay-Italic%5Bwght%5D.ttf" "PlayfairDisplay-Italic.ttf"

echo "=== Downloading Noto CJK/Arabic/Hebrew for internationalization ==="
# Noto Sans JP (Japanese)
download "$GF/ofl/notosansjp/NotoSansJP%5Bwght%5D.ttf" "NotoSansJP-Regular.ttf"
# Noto Sans SC (Simplified Chinese)
download "$GF/ofl/notosanssc/NotoSansSC%5Bwght%5D.ttf" "NotoSansSC-Regular.ttf"
# Noto Sans KR (Korean)
download "$GF/ofl/notosanskr/NotoSansKR%5Bwght%5D.ttf" "NotoSansKR-Regular.ttf"
# Noto Sans Arabic
download "$GF/ofl/notosansarabic/NotoSansArabic%5Bwdth%2Cwght%5D.ttf" "NotoSansArabic-Regular.ttf"
# Noto Sans Hebrew
download "$GF/ofl/notosanshebrew/NotoSansHebrew%5Bwdth%2Cwght%5D.ttf" "NotoSansHebrew-Regular.ttf"
# Noto Sans Devanagari (Hindi)
download "$GF/ofl/notosansdevanagari/NotoSansDevanagari%5Bwdth%2Cwght%5D.ttf" "NotoSansDevanagari-Regular.ttf"

echo ""
echo "=== Done ==="
ls -1 "$FONTS_DIR"/*.ttf 2>/dev/null | wc -l | xargs echo "Total font files:"
