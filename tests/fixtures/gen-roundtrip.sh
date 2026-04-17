#!/usr/bin/env bash
# Generate the roundtrip ANSI test fixture.
# Each line exercises a specific SGR code path.
# Output is written to stdout — redirect to roundtrip.ansi.
set -euo pipefail

ESC='\033'
RST="${ESC}[0m"

# Line 1: Bold text
printf "${ESC}[1mBold text${RST}\n"

# Line 2: Dim text
printf "${ESC}[2mDim text${RST}\n"

# Line 3: Italic text
printf "${ESC}[3mItalic text${RST}\n"

# Line 4: Underline text
printf "${ESC}[4mUnderline text${RST}\n"

# Line 5: Inverse text
printf "${ESC}[7mInverse text${RST}\n"

# Line 6: FG indexed standard (red = 31)
printf "${ESC}[31mRed foreground${RST}\n"

# Line 7: BG indexed standard (green bg = 42)
printf "${ESC}[42mGreen background${RST}\n"

# Line 8: FG indexed extended (color 208 = orange)
printf "${ESC}[38;5;208mOrange 256-color${RST}\n"

# Line 9: BG indexed extended (color 33 = blue)
printf "${ESC}[48;5;33mBlue 256-bg${RST}\n"

# Line 10: FG RGB (teal = 0,180,160)
printf "${ESC}[38;2;0;180;160mTeal RGB fg${RST}\n"

# Line 11: BG RGB (purple = 128,0,255)
printf "${ESC}[48;2;128;0;255mPurple RGB bg${RST}\n"

# Line 12: Bold + FG color (bold + cyan)
printf "${ESC}[1;36mBold cyan${RST}\n"

# Line 13: Italic + underline + BG color
printf "${ESC}[3;4;43mItalic underline yellow-bg${RST}\n"

# Line 14: Multiple attrs + FG + BG
printf "${ESC}[1;4;31;44mBold underline red-on-blue${RST}\n"

# Line 15: Plain text (no styling)
printf "Plain unstyled line\n"

# Line 16: Mixed styled and unstyled on same line
printf "Normal ${ESC}[1mBOLD${RST} normal ${ESC}[3mITALIC${RST} end\n"

# Line 17: FG indexed standard (all 8 basic colors in sequence)
printf "${ESC}[30m0${ESC}[31m1${ESC}[32m2${ESC}[33m3${ESC}[34m4${ESC}[35m5${ESC}[36m6${ESC}[37m7${RST}\n"

# Line 18: Inverse + RGB fg
printf "${ESC}[7;38;2;255;128;0mInverse orange-fg${RST}\n"
